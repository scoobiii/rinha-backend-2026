#!/usr/bin/env bash
# Arquivo:      build_and_push.sh
# Diretório:    /scripts
# Responsável:  Mex — GOS3 · MEx Energia
# Versão:       1.0.0
# Data:         2026-07-06
# Assinatura:   GOS3 · Gang of Seven Senior Scrum Team
# Glossário:    ver docs/GLOSSARIO.md
#
# build_and_push.sh — builda a imagem, publica no GHCR, captura o digest
# imutável e prepara a atualização da branch `submission`.
#
# Uso: GHCR_USER=seu_user GHCR_REPO=rinha-backend-2026 ./scripts/build_and_push.sh
set -euo pipefail

GHCR_USER="${GHCR_USER:?defina GHCR_USER}"
GHCR_REPO="${GHCR_REPO:?defina GHCR_REPO}"
TAG="${TAG:-latest}"
IMAGE="ghcr.io/${GHCR_USER}/${GHCR_REPO}:${TAG}"

# Preferir podman em ARM64/Termux (mais leve que docker completo)
BUILDER="${BUILDER:-podman}"
command -v "$BUILDER" >/dev/null 2>&1 || BUILDER="docker"

echo "== build_and_push (builder: ${BUILDER}) =="

echo "1/4 build"
"$BUILDER" build --platform linux/amd64 -t "$IMAGE" .

echo "2/4 push"
"$BUILDER" push "$IMAGE"

echo "3/4 capturando digest imutável"
DIGEST=$("$BUILDER" inspect --format='{{index .RepoDigests 0}}' "$IMAGE" 2>/dev/null \
  || "$BUILDER" images --digests --format '{{.Digest}}' "$IMAGE" | head -n1)

if [ -z "$DIGEST" ]; then
  echo "Não consegui capturar o digest automaticamente. Rode:"
  echo "  ${BUILDER} inspect ${IMAGE}"
  exit 1
fi

echo "Digest: ${DIGEST}"

echo "4/4 gravando referência para a branch submission"
echo "${DIGEST}" > .submission_digest
echo "Escrito em .submission_digest — atualize a branch submission com esse valor fixo."

echo ""
echo "Próximo passo manual/automatizado:"
echo "  git checkout submission"
echo "  # editar manifest/compose para referenciar: ${DIGEST}"
echo "  git commit -am 'submission: atualiza digest ${DIGEST}'"
echo "  git push origin submission"
