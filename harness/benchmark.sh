#!/usr/bin/env bash
# Arquivo:      benchmark.sh
# Diretório:    /harness
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       1.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# benchmark.sh — mede P99 local. NÃO bate com o preview oficial da Rinha
# (ambiente diferente) — serve só para comparar nossas próprias tentativas
# entre si, de forma relativa. Fonte de verdade real = preview oficial.
set -euo pipefail

API_HOST="${API_HOST:-localhost}"
API_PORT="${API_PORT:-9999}"  # v3: API nao fala mais TCP direto, unico TCP exposto e o do LB
BASE_URL="http://${API_HOST}:${API_PORT}/fraud-score"
REQUESTS="${REQUESTS:-2000}"
CONCURRENCY="${CONCURRENCY:-50}"
LOG_FILE="harness/baseline.json"

command -v hey >/dev/null 2>&1 || {
  echo "Instale 'hey' (https://github.com/rakyll/hey) ou adapte para wrk/vegeta." >&2
  exit 1
}

payload='{"transaction":{"amount":350.00,"installments":1,"requested_at":"2026-07-06T14:30:00Z","card_present":true,"is_online":false},"customer":{"avg_amount":300.00,"tx_count_24h":3,"known_merchants":["merchant-001"]},"merchant":{"id":"merchant-001","mcc":"5411","avg_amount":250.00},"last_transaction":{"minutes_since":120,"km_from_current":2.5}}'

echo "== benchmark local: ${REQUESTS} requests, concorrência ${CONCURRENCY} =="

result=$(hey -n "$REQUESTS" -c "$CONCURRENCY" -m POST \
  -H "Content-Type: application/json" \
  -d "$payload" \
  "$BASE_URL")

echo "$result"

p99=$(echo "$result" | grep "99% in" | awk '{print $3}')
echo "P99 local: ${p99}"

# Log estruturado para comparação entre tentativas (append-only)
timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

entry=$(cat <<EOF
{"timestamp":"${timestamp}","commit":"${commit}","p99_local":"${p99}","requests":${REQUESTS},"concurrency":${CONCURRENCY}}
EOF
)

echo "$entry" >> "$LOG_FILE"
echo "Logado em $LOG_FILE"
echo ""
echo "LEMBRETE: este número é relativo, não compare diretamente com o preview oficial."
