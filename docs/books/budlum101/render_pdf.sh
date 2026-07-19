#!/usr/bin/env bash
set -euo pipefail
# Reproducible local PDF render. Requires pandoc + weasyprint (or wkhtmltopdf).
ROOT="$(cd "$(dirname "$0")" && pwd)"
SOURCE="$ROOT/BUDLUM101_TR.md"
CSS="$ROOT/assets/budlum101.css"
OUT="$ROOT/BUDLUM101_TR.pdf"
command -v pandoc >/dev/null || { echo "pandoc gerekli" >&2; exit 1; }
if command -v weasyprint >/dev/null; then
  pandoc "$SOURCE" --from markdown --standalone --css "$CSS" --pdf-engine=weasyprint -o "$OUT"
elif command -v wkhtmltopdf >/dev/null; then
  pandoc "$SOURCE" --from markdown --standalone --css "$CSS" --pdf-engine=wkhtmltopdf -o "$OUT"
else
  echo "weasyprint veya wkhtmltopdf gerekli" >&2; exit 1
fi
