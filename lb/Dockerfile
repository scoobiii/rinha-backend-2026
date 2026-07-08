# Arquivo:      Dockerfile
# Diretório:    /lb
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       3.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# Build multi-stage: sem arquivos de teste/fixture na imagem final.
# v3.0.0: LB não faz mais proxy de bytes — entrega o fd do cliente aos
# workers via SCM_RIGHTS num socket Unix compartilhado (ver
# docker-compose.yml, volume "sockets").

FROM gcc:13-bookworm AS builder
WORKDIR /build
COPY lb.c .
RUN gcc -O3 -Wall -Wextra -o lb lb.c

FROM debian:bookworm-slim
COPY --from=builder /build/lb /usr/local/bin/lb
EXPOSE 9999
ENTRYPOINT ["/usr/local/bin/lb"]
