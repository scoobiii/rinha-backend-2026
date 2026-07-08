// Arquivo:      main.rs
// Diretório:    /api/src
// Responsável:  Mex — GOS3 · MEx Energia
// Versão:       3.0.0
// Data:         2026-07-06
// Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
// Glossário:    ver docs/GLOSSARIO.md
//
// v3.0.0 — fecha os gaps do SWOT contra o top 5 do leaderboard oficial:
//   - KNN por força bruta O(n)  -> kd-tree particionado (kdtree.rs)
//   - sem SIMD                  -> distância euclidiana em AVX2 (simd.rs)
//   - thread-per-conexão        -> epoll event loop
//   - LB fazia proxy de bytes   -> este worker RECEBE o fd do cliente via
//                                  SCM_RIGHTS (ver lb/lb.c), não abre mais
//                                  seu próprio socket de escuta TCP.
//
// AVISO DE HONESTIDADE TÉCNICA (ver conversa/CONSTRAINTS.md): este arquivo
// NÃO PÔDE ser compilado neste ambiente (sandbox sem rustc instalado e sem
// acesso à rede para buscar as crates). O código foi escrito com cuidado
// e a lógica de negócio (vetorização, kd-tree, AVX2) segue a mesma que já
// rodava na v2.0.0 testada — mas a parte de epoll/SCM_RIGHTS é nova e
// PRECISA ser validada com `cargo build` e os testes do harness antes de
// qualquer submissão. Não trate isso como testado só porque está aqui.
//
// Nomes exatos de campo do payload (transaction/customer/merchant/
// last_transaction) continuam como inferência a partir da documentação —
// ver aviso equivalente que já existia na v2.0.0, mesmo cuidado se aplica.

mod kdtree;
mod simd;

use kdtree::KdTree;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::os::unix::io::RawFd;

pub const DIMS: usize = 16; // 14 dimensões reais do desafio + 2 de padding (sempre 0.0) para alinhar com AVX2 (2x8 lanes)
const K_NEIGHBORS: usize = 5;
const APPROVAL_THRESHOLD: f64 = 0.6;
const SENTINEL_NO_LAST_TX: f32 = -1.0;
const MAX_EVENTS: usize = 256;

// ---------- Dataset de referência (real, carregado do disco) ----------

#[derive(Deserialize)]
struct ReferenceRecord {
    vector: [f32; 14], // dataset oficial tem 14 dims — padding é feito ao carregar
    label: String,     // CONFERIR: nome exato do campo em references.json.gz
}

#[derive(Deserialize)]
struct Normalization {
    max_amount: f64,
    max_installments: f64,
    amount_vs_avg_ratio: f64,
    max_minutes: f64,
    max_km: f64,
    max_tx_count_24h: f64,
    max_merchant_avg_amount: f64,
}

struct Dataset {
    vectors: Vec<[f32; DIMS]>,
    is_fraud: Vec<bool>,
    mcc_risk: HashMap<String, f64>,
    norm: Normalization,
    tree: KdTree,
}

fn pad(v: [f32; 14]) -> [f32; DIMS] {
    let mut out = [0f32; DIMS];
    out[..14].copy_from_slice(&v);
    out
}

fn resources_dir() -> String {
    std::env::var("RESOURCES_DIR").unwrap_or_else(|_| "resources".to_string())
}

fn load_dataset() -> Dataset {
    let dir = resources_dir();

    let norm_path = format!("{}/normalization.json", dir);
    let norm_raw = std::fs::read_to_string(&norm_path)
        .unwrap_or_else(|e| panic!("falha ao ler {}: {}", norm_path, e));
    let norm: Normalization = serde_json::from_str(&norm_raw)
        .unwrap_or_else(|e| panic!("falha ao parsear {}: {}", norm_path, e));

    let mcc_path = format!("{}/mcc_risk.json", dir);
    let mcc_raw = std::fs::read_to_string(&mcc_path)
        .unwrap_or_else(|e| panic!("falha ao ler {}: {}", mcc_path, e));
    let mcc_risk: HashMap<String, f64> = serde_json::from_str(&mcc_raw)
        .unwrap_or_else(|e| panic!("falha ao parsear {}: {}", mcc_path, e));

    let refs_path = format!("{}/references.json.gz", dir);
    let refs_file = File::open(&refs_path).unwrap_or_else(|e| {
        panic!(
            "falha ao abrir {}: {}. Rode scripts/download_dataset.sh primeiro.",
            refs_path, e
        )
    });
    let mut decoder = flate2::read::GzDecoder::new(refs_file);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .unwrap_or_else(|e| panic!("falha ao descomprimir {}: {}", refs_path, e));

    let records: Vec<ReferenceRecord> = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("falha ao parsear dataset descomprimido: {}", e));

    eprintln!("dataset carregado: {} vetores de referência", records.len());

    let mut vectors = Vec::with_capacity(records.len());
    let mut is_fraud = Vec::with_capacity(records.len());
    for r in records {
        vectors.push(pad(r.vector));
        is_fraud.push(r.label == "fraud");
    }

    eprintln!("construindo kd-tree...");
    let tree = KdTree::build(&vectors);
    eprintln!("kd-tree pronta.");

    Dataset {
        vectors,
        is_fraud,
        mcc_risk,
        norm,
        tree,
    }
}

// ---------- Vetorização do payload de entrada (ver AVISO no topo) ----------

fn clamp01(x: f64) -> f32 {
    x.max(0.0).min(1.0) as f32
}

/// Sakamoto's algorithm — dia da semana sem depender de crate de datas.
/// Retorna 0=segunda ... 6=domingo.
fn weekday_mon0(year: i32, month: u32, day: u32) -> u32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    let w = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day as i32).rem_euclid(7);
    ((w + 6) % 7) as u32
}

fn parse_iso8601(ts: &str) -> Option<(u32, u32)> {
    if ts.len() < 19 {
        return None;
    }
    let year: i32 = ts.get(0..4)?.parse().ok()?;
    let month: u32 = ts.get(5..7)?.parse().ok()?;
    let day: u32 = ts.get(8..10)?.parse().ok()?;
    let hour: u32 = ts.get(11..13)?.parse().ok()?;
    Some((hour, weekday_mon0(year, month, day)))
}

fn vectorize(payload: &Value, norm: &Normalization, mcc_risk: &HashMap<String, f64>) -> Option<[f32; DIMS]> {
    let tx = payload.get("transaction")?;
    let customer = payload.get("customer")?;
    let merchant = payload.get("merchant")?;

    let amount = tx.get("amount")?.as_f64()?;
    let installments = tx.get("installments").and_then(Value::as_f64).unwrap_or(1.0);
    let requested_at = tx.get("requested_at").and_then(Value::as_str).unwrap_or("");

    let customer_avg_amount = customer.get("avg_amount").and_then(Value::as_f64).unwrap_or(0.0);
    let tx_count_24h = customer.get("tx_count_24h").and_then(Value::as_f64).unwrap_or(0.0);
    let known_merchants = customer
        .get("known_merchants")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let merchant_id = merchant.get("id").and_then(Value::as_str).unwrap_or("");
    let mcc = merchant.get("mcc").and_then(Value::as_str).unwrap_or("");
    let merchant_avg_amount = merchant.get("avg_amount").and_then(Value::as_f64).unwrap_or(0.0);

    let (hour, weekday) = parse_iso8601(requested_at).unwrap_or((0, 0));

    let unknown_merchant = if known_merchants.iter().any(|m| m.as_str() == Some(merchant_id)) {
        0.0
    } else {
        1.0
    };

    let mcc_risk_score = mcc_risk.get(mcc).copied().unwrap_or(0.5);

    let mut v = [0f32; DIMS];
    v[0] = clamp01(amount / norm.max_amount);
    v[1] = clamp01(installments / norm.max_installments);
    v[2] = clamp01(hour as f64 / 23.0);
    v[3] = clamp01(weekday as f64 / 6.0);
    v[4] = clamp01(if customer_avg_amount > 0.0 {
        (amount / customer_avg_amount) / norm.amount_vs_avg_ratio
    } else {
        1.0
    });

    match payload.get("last_transaction") {
        Some(Value::Object(lt)) => {
            let minutes_since = lt.get("minutes_since").and_then(Value::as_f64).unwrap_or(norm.max_minutes);
            let km_from_current = lt.get("km_from_current").and_then(Value::as_f64).unwrap_or(0.0);
            v[5] = clamp01(minutes_since / norm.max_minutes);
            v[6] = clamp01(km_from_current / norm.max_km);
        }
        _ => {
            v[5] = SENTINEL_NO_LAST_TX;
            v[6] = SENTINEL_NO_LAST_TX;
        }
    }

    v[7] = clamp01(tx_count_24h / norm.max_tx_count_24h);
    v[8] = unknown_merchant;
    v[9] = clamp01(mcc_risk_score);
    v[10] = clamp01(merchant_avg_amount / norm.max_merchant_avg_amount);
    v[11] = clamp01(if merchant_avg_amount > 0.0 {
        amount / merchant_avg_amount / norm.amount_vs_avg_ratio
    } else {
        1.0
    });
    v[12] = if tx.get("card_present").and_then(Value::as_bool).unwrap_or(true) { 0.0 } else { 1.0 };
    v[13] = if tx.get("is_online").and_then(Value::as_bool).unwrap_or(false) { 1.0 } else { 0.0 };
    // v[14], v[15]: padding, permanecem 0.0

    Some(v)
}

/// Busca vetorial real via kd-tree (poda por plano de corte, não é
/// varredura O(n)). fraud_score = fração de vizinhos rotulados fraude.
fn knn_fraud_score(query: &[f32; DIMS], dataset: &Dataset) -> f64 {
    let neighbors = dataset.tree.knn(query, &dataset.vectors, K_NEIGHBORS);
    let frauds = neighbors.iter().filter(|(_, idx)| dataset.is_fraud[*idx]).count();
    frauds as f64 / K_NEIGHBORS as f64
}

// ---------- HTTP mínimo sobre um fd já conectado ----------

fn find_header_end(buf: &[u8]) -> Option<usize> {
    let sep = b"\r\n\r\n";
    buf.windows(sep.len()).position(|w| w == sep).map(|p| p + sep.len())
}

fn content_length(buf: &[u8]) -> Option<usize> {
    let text = String::from_utf8_lossy(buf);
    for line in text.split("\r\n") {
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

fn request_complete(buf: &[u8]) -> bool {
    match find_header_end(buf) {
        Some(header_end) => match content_length(buf) {
            Some(len) => buf.len() >= header_end + len,
            None => true, // sem corpo esperado (ex: GET /ready)
        },
        None => false,
    }
}

fn status_line(status: u16) -> &'static str {
    match status {
        200 => "200 OK",
        400 => "400 Bad Request",
        404 => "404 Not Found",
        _ => "500 Internal Server Error",
    }
}

fn respond_text(status: u16, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
        status_line(status), body.len(), body
    ).into_bytes()
}

fn respond_json(status: u16, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
        status_line(status), body.len(), body
    ).into_bytes()
}

fn handle_request(request: &[u8], dataset: &Dataset) -> Vec<u8> {
    if request.starts_with(b"GET /ready") {
        return respond_text(200, "ready");
    }
    if !request.starts_with(b"POST /fraud-score") {
        return respond_text(404, "not found");
    }

    let body = match find_header_end(request).map(|pos| &request[pos..]) {
        Some(b) if !b.is_empty() => b,
        _ => return respond_text(400, "bad request: sem corpo"),
    };

    let payload: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return respond_text(400, "bad request: JSON inválido"),
    };

    let vector = match vectorize(&payload, &dataset.norm, &dataset.mcc_risk) {
        Some(v) => v,
        None => return respond_text(400, "bad request: payload incompleto"),
    };

    // Busca vetorial REAL via kd-tree+AVX2 — nunca stub (CONSTRAINTS.md Regra #0).
    let fraud_score = knn_fraud_score(&vector, dataset);
    let approved = fraud_score < APPROVAL_THRESHOLD;

    let json = format!("{{\"approved\":{},\"fraud_score\":{:.4}}}", approved, fraud_score);
    respond_json(200, &json)
}

// ---------- Recepção de fds via SCM_RIGHTS (worker lado do lb/lb.c) ----------

/// Bloqueante: aceita a conexão de controle do LB no socket Unix. O LB
/// conecta uma vez no startup e mantém essa conexão para enviar fds.
fn accept_control_connection(listen_fd: RawFd) -> RawFd {
    unsafe {
        let fd = libc::accept(listen_fd, std::ptr::null_mut(), std::ptr::null_mut());
        if fd < 0 {
            panic!("accept() no socket de controle falhou: {}", std::io::Error::last_os_error());
        }
        fd
    }
}

fn bind_control_socket(path: &str) -> RawFd {
    let _ = std::fs::remove_file(path); // path pode existir de uma run anterior
    unsafe {
        let fd = libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0);
        if fd < 0 {
            panic!("socket(AF_UNIX) falhou: {}", std::io::Error::last_os_error());
        }

        let mut addr: libc::sockaddr_un = mem::zeroed();
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        let c_path = CString::new(path).expect("path do socket com byte nulo");
        let bytes = c_path.as_bytes_with_nul();
        if bytes.len() > addr.sun_path.len() {
            panic!("path do socket Unix muito longo: {}", path);
        }
        for (i, b) in bytes.iter().enumerate() {
            addr.sun_path[i] = *b as libc::c_char;
        }

        let addr_len = mem::size_of::<libc::sockaddr_un>() as libc::socklen_t;
        if libc::bind(fd, &addr as *const _ as *const libc::sockaddr, addr_len) < 0 {
            panic!("bind({}) falhou: {}", path, std::io::Error::last_os_error());
        }
        if libc::listen(fd, 8) < 0 {
            panic!("listen() falhou: {}", std::io::Error::last_os_error());
        }
        fd
    }
}

fn set_nonblocking(fd: RawFd) {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL, 0);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
}

/// Drena todos os fds pendentes recebidos via SCM_RIGHTS no socket de
/// controle (pode haver mais de um por evento de epoll level-triggered).
/// Retorna os novos client fds recebidos.
fn recv_fds(control_fd: RawFd) -> Vec<RawFd> {
    let mut received = Vec::new();

    loop {
        let mut dummy_buf = [0u8; 1];
        let mut iov = libc::iovec {
            iov_base: dummy_buf.as_mut_ptr() as *mut libc::c_void,
            iov_len: dummy_buf.len(),
        };

        let mut cmsg_buf = [0u8; 128]; // espaço suficiente para CMSG_SPACE(sizeof(int))
        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = &mut iov as *mut libc::iovec;
        msg.msg_iovlen = 1;
        msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
        msg.msg_controllen = cmsg_buf.len() as _; // tipo do campo varia por plataforma; `as _` deixa o compilador inferir (ver docs/DEBUG-BUILD.md #1)

        let n = unsafe { libc::recvmsg(control_fd, &mut msg, libc::MSG_DONTWAIT) };
        if n <= 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::WouldBlock {
                // EAGAIN é o caminho normal de "acabaram os pendentes".
                // Outros erros (conexão do LB caiu) só param o loop de drain;
                // o processo continua servindo conexões já aceitas.
            }
            break;
        }

        unsafe {
            let cmsg = libc::CMSG_FIRSTHDR(&msg);
            if !cmsg.is_null()
                && (*cmsg).cmsg_level == libc::SOL_SOCKET
                && (*cmsg).cmsg_type == libc::SCM_RIGHTS
            {
                let data_ptr = libc::CMSG_DATA(cmsg) as *const libc::c_int;
                let client_fd = *data_ptr;
                received.push(client_fd);
            }
        }
    }

    received
}

// ---------- Event loop principal (epoll) ----------

struct ConnState {
    buf: Vec<u8>,
}

fn main() {
    eprintln!("carregando dataset...");
    let dataset = load_dataset();
    eprintln!("dataset pronto ({} vetores).", dataset.vectors.len());

    let socket_path = std::env::var("CONTROL_SOCKET")
        .unwrap_or_else(|_| "/sockets/api.sock".to_string());
    let listen_fd = bind_control_socket(&socket_path);
    eprintln!("aguardando conexão do load balancer em {}", socket_path);
    let control_fd = accept_control_connection(listen_fd);
    set_nonblocking(control_fd);
    eprintln!("load balancer conectado — iniciando event loop (epoll)");

    let epfd = unsafe { libc::epoll_create1(0) };
    if epfd < 0 {
        panic!("epoll_create1 falhou: {}", std::io::Error::last_os_error());
    }

    let mut ev = libc::epoll_event { events: libc::EPOLLIN as u32, u64: control_fd as u64 };
    unsafe {
        libc::epoll_ctl(epfd, libc::EPOLL_CTL_ADD, control_fd, &mut ev);
    }

    let mut connections: HashMap<RawFd, ConnState> = HashMap::new();
    let mut events: Vec<libc::epoll_event> = vec![unsafe { mem::zeroed() }; MAX_EVENTS];

    loop {
        let n = unsafe {
            libc::epoll_wait(epfd, events.as_mut_ptr(), MAX_EVENTS as i32, -1)
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            panic!("epoll_wait falhou: {}", err);
        }

        for i in 0..n as usize {
            let fd = events[i].u64 as RawFd;

            if fd == control_fd {
                // Novos fds de clientes chegando via SCM_RIGHTS.
                for client_fd in recv_fds(control_fd) {
                    set_nonblocking(client_fd);
                    let mut cev = libc::epoll_event {
                        events: libc::EPOLLIN as u32,
                        u64: client_fd as u64,
                    };
                    unsafe {
                        libc::epoll_ctl(epfd, libc::EPOLL_CTL_ADD, client_fd, &mut cev);
                    }
                    connections.insert(client_fd, ConnState { buf: Vec::with_capacity(4096) });
                }
                continue;
            }

            // Evento num client_fd: ler o que estiver disponível.
            let mut chunk = [0u8; 8192];
            let read_n = unsafe {
                libc::read(fd, chunk.as_mut_ptr() as *mut libc::c_void, chunk.len())
            };

            if read_n <= 0 {
                // Conexão fechada ou erro — remove e libera.
                unsafe { libc::epoll_ctl(epfd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut()); }
                unsafe { libc::close(fd); }
                connections.remove(&fd);
                continue;
            }

            let state = connections.entry(fd).or_insert_with(|| ConnState { buf: Vec::new() });
            state.buf.extend_from_slice(&chunk[..read_n as usize]);

            if request_complete(&state.buf) {
                let response = handle_request(&state.buf, &dataset);

                // Escrita simples (blocking best-effort): payloads de
                // resposta são pequenos (JSON curto), cabem no buffer de
                // socket sem precisar de estado de escrita parcial.
                unsafe {
                    libc::write(fd, response.as_ptr() as *const libc::c_void, response.len());
                }

                unsafe { libc::epoll_ctl(epfd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut()); }
                unsafe { libc::close(fd); }
                connections.remove(&fd);
            }
        }
    }
}
