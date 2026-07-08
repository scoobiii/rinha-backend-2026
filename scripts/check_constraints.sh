#!/usr/bin/env bash
# Arquivo:      check_constraints.sh
# Diretório:    /scripts
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       1.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# check_constraints.sh — checagens automatizáveis contra CONSTRAINTS.md.
# Isto NÃO substitui revisão humana da zona cinza — só pega violações óbvias
# e mecânicas. Rodar antes de qualquer commit em `submission`.
set -euo pipefail

FAIL=0

warn() { echo "[VIOLAÇÃO] $1" >&2; FAIL=1; }
ok()   { echo "[OK] $1"; }

echo "== check_constraints =="

# 1. LB não deve conter parsing de JSON/payload
if grep -qiE 'json|parse_payload|serde' lb/lb.c 2>/dev/null; then
  warn "lb/lb.c parece conter parsing de payload — proibido (ver CONSTRAINTS.md)"
else
  ok "lb/lb.c sem parsing de payload aparente"
fi

# Helper: retorna apenas as linhas de CÓDIGO de um arquivo, removendo
# comentários (// , # , * de bloco). Evita falso positivo quando um
# comentário está só EXPLICANDO uma proibição (ex: "proibido calcular
# score aqui"), que não é a mesma coisa que calcular o score de fato.
code_only() {
  grep -vE '^[[:space:]]*(//|#|\*|/\*)' "$1" 2>/dev/null
}

# 2. LB não deve calcular score (checagem só em linhas de código, não comentário)
if code_only lb/lb.c | grep -qiE 'score|fraud'; then
  warn "lb/lb.c parece conter lógica de score/fraude em código (fora de comentário) — proibido"
else
  ok "lb/lb.c sem lógica de score aparente (fora de comentários)"
fi

# 3. API não deve ter lookup por ID / tabela de payloads conhecidos
if grep -qriE 'known_payloads|lookup_table|hardcoded_ids' api/src 2>/dev/null; then
  warn "api/src parece conter lookup/tabela hardcoded — proibido"
else
  ok "api/src sem lookup hardcoded aparente"
fi

# 3b. Política Zero-Stub: proibido mock/stub/placeholder/dummy/fake no
# CÓDIGO (não em comentários que documentam a política) da API e do LB
# quando o dataset oficial está disponível. Regra absoluta (ver
# CONSTRAINTS.md), não zona cinza.
FORBIDDEN_TERMS='stub|placeholder|mock|dummy|fake_index|hardcoded_score'

check_zero_stub() {
  local target_dir="$1"
  local label="$2"
  local hit=""
  while IFS= read -r -d '' f; do
    local match
    match=$(code_only "$f" | grep -viE 'zero-stub' | grep -inE "$FORBIDDEN_TERMS" || true)
    if [ -n "$match" ]; then
      hit="${f}: $(echo "$match" | head -1)"
      break
    fi
  done < <(find "$target_dir" -type f \( -name '*.rs' -o -name '*.c' -o -name '*.h' \) -print0 2>/dev/null)

  if [ -n "$hit" ]; then
    warn "$label contém termo proibido pela Política Zero-Stub em código ($hit) — CONSTRAINTS.md"
  else
    ok "$label sem stub/mock/placeholder em código (fora de comentários)"
  fi
}

check_zero_stub api/src "api/src"
check_zero_stub lb "lb/"

# 3c. Dataset real precisa existir — sem ele, a API não tem o que rodar
# de verdade e qualquer score seria fraude (ver CONSTRAINTS.md, Regra #0).
if [ ! -s resources/references.json.gz ]; then
  warn "resources/references.json.gz ausente ou vazio — rode scripts/download_dataset.sh (ver resources/README.md). Sem o dataset real, build de submissão é bloqueado pela Regra #0."
else
  ok "resources/references.json.gz presente ($(wc -c < resources/references.json.gz) bytes)"
fi

if [ ! -s resources/mcc_risk.json ] || [ ! -s resources/normalization.json ]; then
  warn "resources/mcc_risk.json ou resources/normalization.json ausente"
else
  ok "resources/mcc_risk.json e resources/normalization.json presentes"
fi

# 4. Imagem não deve depender de arquivos de teste
for df in api/Dockerfile lb/Dockerfile; do
  if [ -f "$df" ] && grep -qE '(COPY|ADD).*(test|fixture)' "$df"; then
    warn "$df copia arquivos de teste para a imagem final — proibido"
  else
    ok "$df sem cópia de fixtures aparente (ou ainda não existe)"
  fi
done

# 5. Limite de recursos declarado no compose
if [ -f docker-compose.yml ]; then
  grep -qE 'cpus|mem_limit' docker-compose.yml || \
    warn "docker-compose.yml sem limites de CPU/memória declarados"
else
  echo "[INFO] docker-compose.yml ainda não existe — pular checagem de recursos"
fi

echo ""
if [ "$FAIL" -eq 1 ]; then
  echo "== check_constraints FALHOU — corrigir antes de prosseguir =="
  exit 1
else
  echo "== check_constraints passou (checagens mecânicas apenas — revisar zona cinza manualmente) =="
fi
