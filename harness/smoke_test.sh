#!/usr/bin/env bash
# Arquivo:      smoke_test.sh
# Diretório:    /harness
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       2.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# smoke_test.sh — valida o contrato REAL do desafio: GET /ready,
# POST /fraud-score com resposta {"approved":bool,"fraud_score":number}.
# v2.0.0: corrige /health -> /ready e o formato de resposta (era {"score"},
# contrato oficial é {"approved","fraud_score"}).
set -euo pipefail

API_HOST="${API_HOST:-localhost}"
API_PORT="${API_PORT:-9999}"  # v3: API nao fala mais TCP direto, unico TCP exposto e o do LB
BASE_URL="http://${API_HOST}:${API_PORT}"

fail() { echo "FALHOU: $1" >&2; exit 1; }

echo "== smoke test =="

# 1. Ready check (NÃO é /health — ver CONSTRAINTS.md)
status=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/ready") || fail "conexão recusada"
[ "$status" = "200" ] || fail "GET /ready retornou $status, esperado 200"
echo "OK  GET /ready -> 200"

# 2. Payload de transação — CONFERIR nomes de campo exatos contra
# docs/br/API.md do repositório oficial antes de submeter (ver comentário
# de honestidade técnica no topo de api/src/main.rs).
payload='{
  "transaction": {"amount": 350.00, "installments": 1, "requested_at": "2026-07-06T14:30:00Z", "card_present": true, "is_online": false},
  "customer": {"avg_amount": 300.00, "tx_count_24h": 3, "known_merchants": ["merchant-001"]},
  "merchant": {"id": "merchant-001", "mcc": "5411", "avg_amount": 250.00},
  "last_transaction": {"minutes_since": 120, "km_from_current": 2.5}
}'

resp=$(curl -s -X POST "${BASE_URL}/fraud-score" \
  -H "Content-Type: application/json" \
  -d "$payload") || fail "POST /fraud-score sem resposta"

echo "$resp" | grep -q '"approved"' || fail "resposta sem campo 'approved': $resp"
echo "$resp" | grep -q '"fraud_score"' || fail "resposta sem campo 'fraud_score': $resp"
echo "OK  POST /fraud-score -> $resp"

# 3. fraud_score real deve variar com o input — testar um segundo payload
# bem diferente do primeiro para confirmar que a resposta NÃO é fixa
# (detecta regressão pra stub, ver CONSTRAINTS.md Regra #0).
payload2='{
  "transaction": {"amount": 9999.00, "installments": 12, "requested_at": "2026-07-06T03:15:00Z", "card_present": false, "is_online": true},
  "customer": {"avg_amount": 50.00, "tx_count_24h": 19, "known_merchants": []},
  "merchant": {"id": "merchant-999", "mcc": "7995", "avg_amount": 40.00},
  "last_transaction": null
}'

resp2=$(curl -s -X POST "${BASE_URL}/fraud-score" \
  -H "Content-Type: application/json" \
  -d "$payload2") || fail "POST /fraud-score (payload2) sem resposta"

echo "OK  POST /fraud-score (payload de alto risco) -> $resp2"

if [ "$resp" = "$resp2" ]; then
  fail "ALERTA DE FRAUDE: dois payloads muito diferentes deram a MESMA resposta — suspeita de stub/valor fixo (ver CONSTRAINTS.md Regra #0)"
fi
echo "OK  respostas diferentes para inputs diferentes — não é stub"

echo "== smoke test passou =="
