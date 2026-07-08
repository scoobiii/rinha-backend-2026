<!--
Arquivo:      DEVOPS.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       3.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Guia DevOps — Deps, Build, Deploy, Submissão

## 1. Dependências de desenvolvimento

| Ferramenta | Uso | Instalação (Ubuntu/proot) |
|---|---|---|
| `rustc`/`cargo` | build da API | `curl https://sh.rustup.rs -sSf \| sh` |
| `gcc`/`clang` | build do load balancer | `apt install build-essential` |
| `podman` (preferido) ou `docker` | build/run de imagens | `apt install podman` |
| `git` | versionamento e branch `submission` | `apt install git` |
| `hey` | benchmark de carga | `go install github.com/rakyll/hey@latest` |
| `jq` (opcional) | inspecionar `baseline.json` | `apt install jq` |

Em ARM64/Termux: `podman` é preferível a `docker` completo (menor overhead).
Build de imagem para `linux/amd64` (arquitetura de submissão da Rinha) a
partir de ARM64 exige `--platform linux/amd64` com QEMU emulation, ou usar
GitHub Actions como cross-compiler (ver `.github/workflows/`).

## 2. Dataset real (obrigatório — ver CONSTRAINTS.md Regra #0)

```bash
bash scripts/download_dataset.sh
```

Baixa `resources/references.json.gz` (100.000 vetores rotulados) e
`resources/example-references.json` do repositório oficial, e confere
`mcc_risk.json`/`normalization.json` locais contra o upstream. Sem isso, o
build passa mas o container derruba no startup (comportamento esperado —
nunca substituir por mock/stub).

## 3. Build local (sem container)

```bash
# API (precisa de resources/ presente para rodar, não para compilar)
cd api && cargo build --release

# Load balancer
cd ../lb && gcc -O3 -o lb lb.c
```

## 4. Build de imagem

```bash
export GHCR_USER=seu_usuario
export GHCR_REPO=rinha-backend-2026
bash scripts/build_and_push.sh
```

Isso builda, publica no GHCR e grava o digest imutável em
`.submission_digest`.

## 5. Ciclo de submissão

0. **`cd api && cargo build --release && cargo test`** — obrigatório antes
   de qualquer outro passo na v3.0.0: o código Rust (epoll/SCM_RIGHTS/
   kd-tree/AVX2) foi escrito num ambiente sem `rustc` disponível e nunca
   foi compilado. Isso não é opcional aqui.
1. `bash scripts/download_dataset.sh` — dataset real presente
2. `bash scripts/check_constraints.sh` — sem violação mecânica (inclui checagem de stub/mock/placeholder e presença do dataset)
3. `bash harness/smoke_test.sh` — contrato ok E resposta não é idêntica entre payloads diferentes (anti-stub)
4. `bash harness/benchmark.sh` — comparar com baseline local anterior
5. `bash scripts/build_and_push.sh` — captura digest
6. Atualizar branch `submission` com o digest fixo
7. Abrir issue de comparação no repositório oficial da Rinha
8. Acompanhar resultado no leaderboard oficial (fonte de verdade real)
9. **Antes de tudo isso: confirmar em rinhadebackend.com.br se a edição
   ainda está recebendo submissões** — houve sinal conflitante sobre o
   status no momento em que este repo foi escrito (ver topo de `CONSTRAINTS.md`)

## 6. CI (GitHub Actions)

Ver `.github/workflows/ci.yml` — roda `check_constraints.sh` e
`smoke_test.sh` a cada push, cross-compila para `amd64`.

## 7. Rollback

Todo digest fica registrado em `harness/baseline.json` (campo `commit`) e
em `.submission_digest` no momento do build. Para reverter:

```bash
git checkout submission
# editar manifest/compose para o digest anterior salvo no baseline.json
git commit -am "rollback: volta para digest anterior"
git push origin submission
```

## 8. Troubleshooting

| Sintoma | Causa provável | Solução |
|---|---|---|
| Build falha em ARM64 para `amd64` | Falta QEMU/binfmt | `apt install qemu-user-static binfmt-support` |
| `check_constraints.sh` falha | Violação mecânica detectada | Ler a saída, corrigir antes de prosseguir — nunca ignorar |
| Digest não capturado | Builder não suporta `inspect --format` | Rodar `podman inspect <imagem>` manualmente e copiar o digest |
| P99 local não bate com preview oficial | Esperado — ambientes diferentes | Ver Glossário: **Preview** |
