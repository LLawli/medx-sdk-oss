#!/usr/bin/env bash
set -e

git push

echo "Aguardando CI compilar o binário (5 minutos)..."
sleep 300

echo "Baixando última release do medx-cli..."
gh release download latest \
  --repo LLawli/medx-sdk-oss \
  --pattern "medx-cli" \
  --output ~/.local/bin/medx-cli \
  --clobber

chmod +x ~/.local/bin/medx-cli
echo "Instalado: $(medx-cli --version)"
