/*
 * Arquivo:      lb.c
 * Diretório:    /lb
 * Responsável:  Mex — GOS3 · MEx Energia
 * Versão:       3.0.0
 * Data:         2026-07-06
 * Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
 * Glossário:    ver docs/GLOSSARIO.md
 *
 * v3.0.0 — reescrita para fechar o gap identificado no SWOT contra o top 5
 * do leaderboard (ver docs/00-ROADMAP-16-DIAS.md): v1/v2 faziam proxy de
 * bytes (round-robin com cópia dupla). Esta versão faz FD PASSING via
 * SCM_RIGHTS: o LB aceita a conexão do cliente e entrega o file descriptor
 * já conectado para um worker via socket Unix, sem nunca ler/escrever o
 * corpo da requisição. Isso elimina uma cópia inteira de dados e é a mesma
 * técnica citada nos tópicos "scm_rights" das submissões líderes.
 *
 * REGRA DE OURO (ver CONSTRAINTS.md): este arquivo NÃO deve conter parsing
 * de payload, cálculo de score, ou qualquer decisão baseada em conteúdo da
 * requisição. Só round-robin cru + handoff de fd.
 *
 * Arquitetura:
 *   1. epoll_wait no socket de escuta (porta 9999)
 *   2. accept() do cliente -> client_fd
 *   3. round-robin escolhe o socket Unix do próximo worker
 *   4. sendmsg() com ancillary data SCM_RIGHTS entrega client_fd ao worker
 *   5. LB fecha sua cópia do client_fd (worker agora é dono dele)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <sys/epoll.h>
#include <fcntl.h>

#define MAX_WORKERS 16
#define LISTEN_PORT 9999
#define MAX_EVENTS 64

static int worker_fds[MAX_WORKERS]; /* sockets Unix já conectados a cada worker */
static int worker_count = 0;
static int rr_index = 0;

/* Carrega caminhos de socket Unix de LB_WORKER_SOCKETS: "/sockets/api1.sock,/sockets/api2.sock" */
static void connect_workers(void) {
    const char *env = getenv("LB_WORKER_SOCKETS");
    if (!env) {
        fprintf(stderr, "LB_WORKER_SOCKETS não definido. Ex: /sockets/api1.sock,/sockets/api2.sock\n");
        exit(1);
    }
    char buf[1024];
    strncpy(buf, env, sizeof(buf) - 1);
    buf[sizeof(buf) - 1] = '\0';

    char *tok = strtok(buf, ",");
    while (tok && worker_count < MAX_WORKERS) {
        int fd = socket(AF_UNIX, SOCK_STREAM, 0);
        if (fd < 0) { perror("socket(AF_UNIX)"); exit(1); }

        struct sockaddr_un addr;
        memset(&addr, 0, sizeof(addr));
        addr.sun_family = AF_UNIX;
        strncpy(addr.sun_path, tok, sizeof(addr.sun_path) - 1);

        /* Workers podem demorar a subir (carregam dataset + constroem
         * kd-tree no startup) — retry com backoff simples, até ~60s. */
        int connected = 0;
        for (int attempt = 0; attempt < 300 && !connected; attempt++) {
            if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) == 0) {
                connected = 1;
            } else {
                usleep(200000); /* 200ms */
            }
        }
        if (!connected) {
            fprintf(stderr, "falha ao conectar no worker %s\n", tok);
            exit(1);
        }

        worker_fds[worker_count++] = fd;
        fprintf(stderr, "conectado ao worker: %s\n", tok);
        tok = strtok(NULL, ",");
    }

    if (worker_count == 0) {
        fprintf(stderr, "Nenhum worker válido em LB_WORKER_SOCKETS\n");
        exit(1);
    }
}

/* Round-robin cru: apenas avança o índice. Sem inspeção de conteúdo. */
static int next_worker_fd(void) {
    int fd = worker_fds[rr_index];
    rr_index = (rr_index + 1) % worker_count;
    return fd;
}

/* Entrega client_fd ao worker via SCM_RIGHTS (ancillary data). */
static int send_fd(int unix_sock, int fd_to_send) {
    struct msghdr msg = {0};
    struct iovec iov;
    char dummy = 'x'; /* sendmsg exige ao menos 1 byte de payload "normal" */
    char cmsgbuf[CMSG_SPACE(sizeof(int))];

    iov.iov_base = &dummy;
    iov.iov_len = 1;

    msg.msg_iov = &iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsgbuf;
    msg.msg_controllen = sizeof(cmsgbuf);

    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;
    cmsg->cmsg_len = CMSG_LEN(sizeof(int));
    memcpy(CMSG_DATA(cmsg), &fd_to_send, sizeof(int));

    return sendmsg(unix_sock, &msg, 0);
}

static int set_nonblocking(int fd) {
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags < 0) return -1;
    return fcntl(fd, F_SETFL, flags | O_NONBLOCK);
}

int main(void) {
    connect_workers();

    int listen_fd = socket(AF_INET, SOCK_STREAM, 0);
    int opt = 1;
    setsockopt(listen_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(LISTEN_PORT);

    if (bind(listen_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        perror("bind");
        return 1;
    }
    listen(listen_fd, 4096);
    set_nonblocking(listen_fd);

    int epfd = epoll_create1(0);
    if (epfd < 0) { perror("epoll_create1"); return 1; }

    struct epoll_event ev = {0};
    ev.events = EPOLLIN;
    ev.data.fd = listen_fd;
    epoll_ctl(epfd, EPOLL_CTL_ADD, listen_fd, &ev);

    struct epoll_event events[MAX_EVENTS];

    fprintf(stderr, "LB (epoll + scm_rights) ouvindo na porta %d, %d workers\n",
            LISTEN_PORT, worker_count);

    for (;;) {
        int n = epoll_wait(epfd, events, MAX_EVENTS, -1);
        if (n < 0) {
            if (errno == EINTR) continue;
            perror("epoll_wait");
            break;
        }

        for (int i = 0; i < n; i++) {
            if (events[i].data.fd != listen_fd) continue;

            /* Nível de disparo: drena todos os accepts pendentes. */
            for (;;) {
                int client_fd = accept(listen_fd, NULL, NULL);
                if (client_fd < 0) {
                    if (errno == EAGAIN || errno == EWOULDBLOCK) break;
                    if (errno == EINTR) continue;
                    perror("accept");
                    break;
                }

                /* TCP_NODELAY: não é lógica de fraude, é tuning de socket. */
                int one = 1;
                setsockopt(client_fd, IPPROTO_TCP, TCP_NODELAY, &one, sizeof(one));

                int worker_fd = next_worker_fd();
                if (send_fd(worker_fd, client_fd) < 0) {
                    perror("send_fd");
                    /* Round-robin cru: se um worker falhar, não decidimos
                     * por conteúdo — apenas fechamos e seguimos (o próximo
                     * accept vai para o próximo worker naturalmente). */
                }
                close(client_fd); /* worker agora é dono do fd */
            }
        }
    }

    return 0;
}
