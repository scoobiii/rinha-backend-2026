# Arquivo:      Dockerfile
# Diretório:    /api
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       3.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# Build multi-stage: sem arquivos de TESTE na imagem final (ver
# CONSTRAINTS.md). O dataset real (resources/) é montado como volume em
# runtime (ver docker-compose.yml) em vez de ir dentro da imagem — os três
# arquivos não mudam durante o teste, então o pré-processamento acontece no
# startup do container (carregamento em memória), não em build time.
#
# v3.0.0: a API não escuta mais TCP — recebe fds já conectados do LB via
# SCM_RIGHTS num socket Unix (CONTROL_SOCKET). O socket é criado no volume
# compartilhado /sockets (ver docker-compose.yml).
#
# Isso NÃO é a mesma coisa que "arquivo de teste embutido" (proibido): é o
# dataset oficial de referência que a busca vetorial precisa para funcionar
# de verdade — ver CONSTRAINTS.md, Regra #0 (nunca mock/stub/placeholder).

FROM rust:1.79-slim AS builder
WORKDIR /build
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /build/target/release/fraud-score-api /usr/local/bin/api
ENV RESOURCES_DIR=/app/resources
ENV CONTROL_SOCKET=/sockets/api.sock
ENTRYPOINT ["/usr/local/bin/api"]
