#!/bin/bash
# latest.json を手動で作成するスクリプト

VERSION="1.4.0"
REPO="fablab-westharima/DigiCode-Finder"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

# 一時ディレクトリ
mkdir -p /tmp/sigs
cd /tmp/sigs

echo "Downloading signature files..."

# 署名ファイルをダウンロード
curl -sLO "${BASE_URL}/DigiCode.Finder_macOS-Intel_${VERSION}.app.tar.gz.sig" || echo "macOS Intel sig not found"
curl -sLO "${BASE_URL}/DigiCode.Finder_macOS-ARM_${VERSION}.app.tar.gz.sig" || echo "macOS ARM sig not found"
curl -sLO "${BASE_URL}/DigiCode.Finder_Linux_${VERSION}.AppImage.sig" || echo "Linux sig not found"
curl -sLO "${BASE_URL}/DigiCode.Finder_Windows_${VERSION}-setup.exe.sig" || echo "Windows sig not found"

# 署名を読み取り
DARWIN_X64_SIG=""
DARWIN_ARM_SIG=""
LINUX_SIG=""
WINDOWS_SIG=""

[ -f "DigiCode.Finder_macOS-Intel_${VERSION}.app.tar.gz.sig" ] && DARWIN_X64_SIG=$(cat "DigiCode.Finder_macOS-Intel_${VERSION}.app.tar.gz.sig")
[ -f "DigiCode.Finder_macOS-ARM_${VERSION}.app.tar.gz.sig" ] && DARWIN_ARM_SIG=$(cat "DigiCode.Finder_macOS-ARM_${VERSION}.app.tar.gz.sig")
[ -f "DigiCode.Finder_Linux_${VERSION}.AppImage.sig" ] && LINUX_SIG=$(cat "DigiCode.Finder_Linux_${VERSION}.AppImage.sig")
[ -f "DigiCode.Finder_Windows_${VERSION}-setup.exe.sig" ] && WINDOWS_SIG=$(cat "DigiCode.Finder_Windows_${VERSION}-setup.exe.sig")

echo ""
echo "Creating latest.json..."

# latest.json を作成
cat > /tmp/latest.json << EOF
{
  "version": "${VERSION}",
  "notes": "DigiCode Finder v${VERSION}",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "darwin-x86_64": {
      "url": "${BASE_URL}/DigiCode.Finder_macOS-Intel_${VERSION}.app.tar.gz",
      "signature": "${DARWIN_X64_SIG}"
    },
    "darwin-aarch64": {
      "url": "${BASE_URL}/DigiCode.Finder_macOS-ARM_${VERSION}.app.tar.gz",
      "signature": "${DARWIN_ARM_SIG}"
    },
    "linux-x86_64": {
      "url": "${BASE_URL}/DigiCode.Finder_Linux_${VERSION}.AppImage",
      "signature": "${LINUX_SIG}"
    },
    "windows-x86_64": {
      "url": "${BASE_URL}/DigiCode.Finder_Windows_${VERSION}-setup.exe",
      "signature": "${WINDOWS_SIG}"
    }
  }
}
EOF

echo ""
echo "=== latest.json ==="
cat /tmp/latest.json
echo ""
echo ""
echo "File saved to: /tmp/latest.json"
echo ""
echo "To upload, run:"
echo "  gh release upload v${VERSION} /tmp/latest.json --repo ${REPO} --clobber"
