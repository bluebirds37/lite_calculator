#!/usr/bin/env bash
# 打包 macOS .app：release 构建 → 组装 bundle → 生成图标 → zip。
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

name="LiteCalculator"
bundle="dist/${name}.app"
exec_name="calculator"
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
arch="$(uname -m)"

cargo build --release

rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp "target/release/${exec_name}" "$bundle/Contents/MacOS/${exec_name}"

if [[ -f packaging/icon.png ]]; then
  iconset="$(mktemp -d)/AppIcon.iconset"
  mkdir -p "$iconset"
  for size in 16 32 128 256 512; do
    sips -z "$size" "$size" packaging/icon.png --out "$iconset/icon_${size}x${size}.png" >/dev/null
    doubled=$((size * 2))
    sips -z "$doubled" "$doubled" packaging/icon.png --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$iconset" -o "$bundle/Contents/Resources/AppIcon.icns"
  icon_key='<key>CFBundleIconFile</key><string>AppIcon</string>'
else
  icon_key=''
fi

cat >"$bundle/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>${name}</string>
  <key>CFBundleDisplayName</key><string>Lite Calculator</string>
  <key>CFBundleExecutable</key><string>${exec_name}</string>
  <key>CFBundleIdentifier</key><string>com.bluebirds37.lite-calculator</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${version}</string>
  <key>CFBundleVersion</key><string>${version}</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  ${icon_key}
  <key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
PLIST

printf 'APPL????' >"$bundle/Contents/PkgInfo"

codesign --force --deep --sign - "$bundle" 2>/dev/null || \
  echo "跳过代码签名（可手动执行 codesign --sign - ${bundle}）"

zip_name="dist/${name}-v${version}-macos-${arch}.zip"
rm -f "$zip_name"
(cd dist && zip -qry "$(basename "$zip_name")" "${name}.app")

echo "产物：$bundle"
echo "压缩包：$zip_name"
