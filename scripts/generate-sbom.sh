#!/usr/bin/env bash
# scripts/generate-sbom.sh — Rust SBOM üretici (Phase 0.40 §1.7)
#
# Bu script CycloneDX formatında SBOM (Software Bill of Materials)
# üretir. ch12 §3.7 mainnet blocker kapsamında; harici audit
# firması için zorunlu teslim kalemi.
#
# Kullanım:
#   ./scripts/generate-sbom.sh
#
# Çıktı: `sbom.cdx.json` (repo root) + `docs/operations/SBOM.md` özeti.
# Format: CycloneDX 1.5 (JSON).
# Kabul kriteri: SBOM dosyası oluşturulabiliyor + JSON parse oluyor.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "[generate-sbom] SBOM üretimi başlatılıyor..."

# 1. cargo-cyclonedx yükle (yoksa veya sürüm pinli değilse).
# Sürüm pinli: CLI bayrakları sürümler arası değişebiliyor (aşağıdaki not),
# kapının deterministik kalması için pin ZORUNLU.
CYCLONEDX_VERSION="0.5.9"
if ! command -v cargo-cyclonedx >/dev/null 2>&1 \
    || ! cargo cyclonedx --version 2>/dev/null | grep -q "$CYCLONEDX_VERSION"; then
    echo "[generate-sbom] cargo-cyclonedx $CYCLONEDX_VERSION (pinli) yükleniyor..."
    cargo install --locked cargo-cyclonedx --version "$CYCLONEDX_VERSION"
fi

# 2. SBOM üret
# Not: cargo-cyclonedx 0.5.x CLI'sında `--output-file` bayrağı YOKTUR
# (main CI run #728: "error: unexpected argument '--output-file' found",
# exit 2). Çıktı, crate'in manifest dizinine
# `--override-filename <ad>` ile verilen ismin TAMAMI + format uzantısı
# olarak yazılır ("sbom.cdx" + json → "sbom.cdx.json"). ci.yml'deki
# artifact adımının beklediği yol budur (path: sbom.cdx.json).
SBOM_FILE="$REPO_ROOT/sbom.cdx.json"
rm -f "$SBOM_FILE"
# --spec-version 1.5: üretilen BOM'un gerçek specVersion alanını SBOM.md'deki
# "CycloneDX 1.5" beyanıyla hizalar (doküman-kod tutarlılık kuralı, Phase 8.5 §8).
cargo cyclonedx --format json --override-filename "sbom.cdx" --spec-version 1.5

# 3. JSON validasyon
if ! python3 -c "import json; json.load(open('$SBOM_FILE'))" 2>/dev/null; then
    echo "[generate-sbom] HATA: SBOM JSON parse edilemedi."
    exit 1
fi

# 4. Boyut ve bileşen sayısı
SBOM_SIZE="$(stat -c%s "$SBOM_FILE" 2>/dev/null || stat -f%z "$SBOM_FILE" 2>/dev/null || echo "?")"
COMPONENT_COUNT="$(python3 -c "import json; print(len(json.load(open('$SBOM_FILE')).get('components', [])))" 2>/dev/null || echo "?")"

# 5. Rapor
DOC="$REPO_ROOT/docs/operations/SBOM.md"
mkdir -p "$(dirname "$DOC")"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
    echo "# SBOM (Software Bill of Materials)"
    echo ""
    echo "**Oluşturulma:** $TIMESTAMP"
    echo "**Araç:** cargo-cyclonedx (https://github.com/CycloneDX/cyclonedx-rust-cargo)"
    echo "**Format:** CycloneDX 1.5 (JSON)"
    echo "**Repo:** lubosruler/budlum @ \`$(git rev-parse --short HEAD)\`"
    echo ""
    echo "## Özet"
    echo ""
    echo "- **SBOM dosyası:** \`sbom.cdx.json\` (boyut: $SBOM_SIZE byte)"
    echo "- **Bileşen sayısı:** $COMPONENT_COUNT"
    echo ""
    echo "## Kullanım"
    echo ""
    echo "Harici audit firması \`sbom.cdx.json\` dosyasını doğrudan kullanabilir."
    echo "Format: CycloneDX 1.5 JSON, tüm transitive bağımlılıkları içerir."
    echo ""
    echo "## Yenileme"
    echo ""
    echo "\`\`\`bash"
    echo "./scripts/generate-sbom.sh"
    echo "\`\`\`"
    echo ""
    echo "Bu rapor Phase 0.40 §1.7 kapsamında otomatik üretilir."
} > "$DOC"

echo "[generate-sbom] SBOM: $SBOM_FILE ($SBOM_SIZE byte, $COMPONENT_COUNT bileşen)"
echo "[generate-sbom] Rapor: $DOC"
echo "[generate-sbom] Bitti."
