<!--
Arquivo:      README.md
Diretório:    /
Responsável:  Mex — GOS3 · MEx Energia
Versão:       3.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Rinha de Backend 2026 — Fraud Detection + Busca Vetorial

> Metodologia **Akita Vibe Coding**: ver `docs/00-ROADMAP-16-DIAS.md`.
> Guias: `docs/USUARIO.md` (rodar/testar), `docs/DEVOPS.md` (deps/build/deploy).
> **Regra #0 de `CONSTRAINTS.md`: nunca mock, stub ou placeholder no cálculo
> de fraude.** **AVISO: código Rust não foi compilado no ambiente onde este
> repo foi escrito (sem rustc/rede) — rodar `cargo build` antes de confiar
> nele. O `lb.c` foi compilado e testado ponta a ponta de verdade.**

Stack otimizada por **remoção de camadas + estruturas de dados certas**,
com busca vetorial real (kd-tree + AVX2, k=5) sobre o dataset oficial, LB
sem cópia de bytes (SCM_RIGHTS) e workers orientados a evento (epoll) —
fechando o gap identificado contra o top 5 do leaderboard oficial (ver
`CONSTRAINTS.md`, seção "Auto-avaliação técnica").

## Estrutura

```
rinha-backend-2026/
├── CONSTRAINTS.md          # fonte de verdade — checar antes de QUALQUER merge
├── docker-compose.yml      # volume "sockets" (SCM_RIGHTS) + volume resources/ (dataset)
├── resources/
│   ├── README.md
│   ├── normalization.json     # constantes oficiais (versionado)
│   ├── mcc_risk.json          # risco por MCC oficial (versionado)
│   └── references.json.gz     # dataset real (baixado — ver scripts/download_dataset.sh)
├── docs/
│   ├── 00-ROADMAP-16-DIAS.md
│   ├── PROMPTS.md
│   ├── USUARIO.md
│   ├── DEVOPS.md
│   └── GLOSSARIO.md
├── lb/
│   ├── lb.c                # v3: epoll no accept + fd passing via SCM_RIGHTS (testado ponta a ponta)
│   └── Dockerfile
├── api/
│   ├── src/main.rs         # v3: recebe fds via SCM_RIGHTS, epoll event loop
│   ├── src/kdtree.rs       # kd-tree particionado — KNN real, sem força bruta
│   ├── src/simd.rs         # distância euclidiana em AVX2 + fallback escalar
│   ├── Cargo.toml          # serde/serde_json/flate2/libc
│   └── Dockerfile
├── harness/
│   ├── smoke_test.sh       # contrato real (/ready, /fraud-score) + detector anti-stub
│   ├── benchmark.sh
│   └── baseline.json
├── scripts/
│   ├── download_dataset.sh
│   ├── check_constraints.sh
│   └── build_and_push.sh
└── .github/workflows/ci.yml
```

## O que foi validado nesta sessão (e o que não foi)

| Componente | Validação real feita |
|---|---|
| `lb.c` | Compilado com `gcc -Wall -Wextra` sem warning. Testado ponta a ponta: cliente real → LB → worker simulado (Python) → resposta de volta pelo mesmo fd. Round-robin confirmado entre 2 workers. |
| `main.rs`, `kdtree.rs`, `simd.rs` | **Não compilados** — sem `rustc` no ambiente de edição. Rodar `cargo build --release` e `cargo test` antes de confiar. |

## Fluxo de trabalho

1. `bash scripts/download_dataset.sh` — dataset real presente (uma vez)
2. `cd api && cargo build --release && cargo test` — **primeira validação real do lado Rust**
3. `scripts/check_constraints.sh` — bloqueia stub/mock/placeholder
4. `harness/smoke_test.sh` — contrato + detector anti-stub
5. `harness/benchmark.sh` — compara com baseline local
6. `scripts/build_and_push.sh` — build, publica, captura digest
7. Atualiza branch `submission`, abre issue de comparação
8. Confirma status da edição em rinhadebackend.com.br antes de submeter

## ARM64 / Termux

`simd.rs` compila e roda em ARM64 (usa o caminho escalar — AVX2 é exclusivo
de x86). O binário final para submissão é cross-compilado para `amd64` via
CI (`--platform linux/amd64`), onde o caminho AVX2 real entra em ação.
