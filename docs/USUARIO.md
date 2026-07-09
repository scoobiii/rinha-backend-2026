<!--
Arquivo:      USUARIO.md
Diretório:    /docs
Responsável:  Mex — GOS3 · MEx Energia
Versão:       3.0.0
Data:         2026-07-06
Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
Glossário:    ver docs/GLOSSARIO.md
-->

# Guia do Usuário — Instalar, Rodar, Testar

Para quem só quer subir a stack localmente e ver funcionando, sem lidar com
build de imagem/publicação (isso é `docs/DEVOPS.md`).

## 1. Pré-requisitos

- `docker` ou `podman` instalado
- `docker-compose` (ou `podman-compose`)
- `curl` (para testar manualmente)
- `curl` com acesso à internet (para baixar o dataset oficial uma vez)

Em Android/Termux: instalar `proot-Ubuntu` primeiro, depois seguir os passos
dentro dele — não instalar direto no Termux puro.

## 2. Instalar

```bash
git clone <url-do-repo>
cd rinha-backend-2026
```

Nenhuma dependência externa de linguagem precisa ser instalada manualmente —
tudo builda dentro dos containers (`api/Dockerfile`, `lb/Dockerfile`).

### 2b. Dataset real (obrigatório antes de rodar)

A API se recusa a iniciar sem o dataset oficial da Rinha (ver
`CONSTRAINTS.md`, Regra #0 — nunca mock/stub/placeholder). Baixar uma vez:

```bash
bash scripts/download_dataset.sh
```

Isso popula `resources/references.json.gz` (1.000.000 vetores rotulados) e
`resources/example-references.json`, além de conferir `mcc_risk.json` e
`normalization.json` contra a fonte oficial. Ver `resources/README.md`.

## 3. Rodar

```bash
docker compose up --build
```

Isso sobe:
- 1 load balancer (`lb`) na porta `9999`
- 2 instâncias de API (`api1`, `api2`) atrás do load balancer, cada uma
  carregando o dataset real de `resources/` (montado como volume)

Para parar:

```bash
docker compose down
```

## 4. Testar

### Teste rápido manual

```bash
curl http://localhost:9999/ready
```

Esperado: `200 OK` (não é `/health` — o contrato oficial da Rinha usa `/ready`).

```bash
curl -X POST http://localhost:9999/fraud-score \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": {"amount": 350.00, "installments": 1, "requested_at": "2026-07-06T14:30:00Z", "card_present": true, "is_online": false},
    "customer": {"avg_amount": 300.00, "tx_count_24h": 3, "known_merchants": ["merchant-001"]},
    "merchant": {"id": "merchant-001", "mcc": "5411", "avg_amount": 250.00},
    "last_transaction": {"minutes_since": 120, "km_from_current": 2.5}
  }'
```

Esperado: JSON com campos `"approved"` (bool) e `"fraud_score"` (número).
**Os nomes de campo do payload precisam ser conferidos contra
`docs/br/API.md` do repositório oficial antes de qualquer submissão** — ver
aviso no topo de `api/src/main.rs`.

### Teste automatizado (contrato + anti-stub)

```bash
API_HOST=localhost API_PORT=9999 bash harness/smoke_test.sh
```

Esse teste manda dois payloads bem diferentes e falha explicitamente se a
resposta for idêntica nos dois — isso pegaria uma regressão para stub.

### Teste de performance local

Requer `hey` instalado (`go install github.com/rakyll/hey@latest` ou pacote
da distro):

```bash
API_HOST=localhost API_PORT=9999 bash harness/benchmark.sh
```

O resultado é só uma referência local — não é o número oficial da Rinha
(ver Glossário: **Preview**).

## 5. Problemas comuns

| Sintoma | Causa provável | Solução |
|---|---|---|
| `connection refused` na porta 9999 | Containers ainda subindo | Aguardar alguns segundos, checar `docker compose logs` |
| API derruba no boot com erro de arquivo ausente | `resources/references.json.gz` não baixado | Rodar `bash scripts/download_dataset.sh` — comportamento esperado, não é bug |
| `curl` no `/fraud-score` retorna 400 "payload incompleto" | Payload sem os campos mínimos (`transaction`, `customer`, `merchant`) | Conferir contra `docs/br/API.md` oficial e o exemplo acima |
| `smoke_test.sh` falha com "ALERTA DE FRAUDE" | Resposta idêntica para inputs diferentes | Sinal de que voltou a ter stub/valor fixo — ver `CONSTRAINTS.md` Regra #0 |
| `hey: command not found` | Ferramenta de benchmark não instalada | Instalar `hey` ou adaptar `benchmark.sh` para `wrk`/`vegeta` |
