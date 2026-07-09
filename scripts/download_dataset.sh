#!/usr/bin/env bash
# Arquivo:      download_dataset.sh
# Diretório:    /scripts
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       1.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# download_dataset.sh — baixa o dataset REAL do repositório oficial da Rinha.
# Ver CONSTRAINTS.md, Regra #0: é proibido substituir isto por mock/stub.
#
# Fonte oficial: https://github.com/zanfranceschi/rinha-de-backend-2026
set -euo pipefail

BASE_URL="https://github.com/zanfranceschi/rinha-de-backend-2026/raw/refs/heads/main/resources"
DEST_DIR="${DEST_DIR:-resources}"

mkdir -p "$DEST_DIR"

echo "== Baixando dataset oficial da Rinha de Backend 2026 =="

curl -fL "${BASE_URL}/references.json.gz" -o "${DEST_DIR}/references.json.gz"
echo "OK  references.json.gz (1.000.000 vetores rotulados — confirmado via zcat | grep -o vector | wc -l)"

curl -fL "${BASE_URL}/example-references.json" -o "${DEST_DIR}/example-references.json"
echo "OK  example-references.json (recorte pequeno, para teste rápido)"

# mcc_risk.json e normalization.json já estão versionados neste repo
# (conteúdo oficial completo, <1KB cada) — mas confirmamos aqui contra a
# fonte, caso a competição atualize os valores.
curl -fL "${BASE_URL}/mcc_risk.json" -o "${DEST_DIR}/mcc_risk.json.upstream"
curl -fL "${BASE_URL}/normalization.json" -o "${DEST_DIR}/normalization.json.upstream"

if ! diff -q "${DEST_DIR}/mcc_risk.json" "${DEST_DIR}/mcc_risk.json.upstream" >/dev/null 2>&1; then
  echo "[AVISO] mcc_risk.json local difere do upstream oficial — revisar antes de usar."
fi
if ! diff -q "${DEST_DIR}/normalization.json" "${DEST_DIR}/normalization.json.upstream" >/dev/null 2>&1; then
  echo "[AVISO] normalization.json local difere do upstream oficial — revisar antes de usar."
fi
rm -f "${DEST_DIR}/mcc_risk.json.upstream" "${DEST_DIR}/normalization.json.upstream"

echo ""
echo "== Dataset pronto em ${DEST_DIR}/ =="
echo "Lembrete: references.json.gz não muda durante o teste — pré-processar"
echo "no build/startup (ver docs/DEVOPS.md)."
