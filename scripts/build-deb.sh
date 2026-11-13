#!/usr/bin/env bash
set -e

# 确保在项目根目录运行
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

# 从 Cargo.toml 自动提取应用名称与版本号
PACKAGE_NAME=$(grep -m1 '^name =' Cargo.toml | cut -d '"' -f2)
VERSION=$(grep -m1 '^version =' Cargo.toml | cut -d '"' -f2)
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")

echo "=================================================="
echo " 构建 Debian 安装包: ${PACKAGE_NAME} v${VERSION} (${ARCH})"
echo "=================================================="

# 1. 编译 Release 二进制文件
echo "==> 正在编译 Release 优化产物..."
cargo build --release

RELEASE_BIN="target/release/${PACKAGE_NAME}"
if [ ! -f "$RELEASE_BIN" ]; then
    echo "错误: 未找到可执行文件: $RELEASE_BIN"
    exit 1
fi

# 2. 准备暂存打包目录
PKG_DIR="target/debian/pkg"
rm -rf "target/debian"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps"

# 3. 拷贝程序二进制与桌面资产
echo "==> 拷贝文件至打包结构..."
cp "$RELEASE_BIN" "${PKG_DIR}/usr/bin/"
chmod 755 "${PKG_DIR}/usr/bin/${PACKAGE_NAME}"

cp "assets/ubuntu-wall.desktop" "${PKG_DIR}/usr/share/applications/"
chmod 644 "${PKG_DIR}/usr/share/applications/ubuntu-wall.desktop"

cp "assets/ubuntu-wall.svg" "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps/"
chmod 644 "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps/ubuntu-wall.svg"

# 4. 计算安装体积 (KB)
INSTALLED_SIZE=$(du -sk "${PKG_DIR}/usr" | cut -f1)

# 5. 生成 control 控制文件
echo "==> 生成 DEBIAN/control 元数据..."
cat << EOF > "${PKG_DIR}/DEBIAN/control"
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Ubuntu Wall Contributors
Depends: libc6, libgtk-3-0, libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37, libappindicator3-1
Description: Ubuntu Multi-Monitor Wallpaper Manager
 A native wallpaper manager for Ubuntu GNOME Wayland & X11.
 Allows setting individual wallpapers for multiple connected monitors
 with virtual canvas stitching and Material Design 3 UI.
EOF

# 6. 生成 postinst 与 postrm 触发器 (更新桌面入口与图标缓存)
cat << 'EOF' > "${PKG_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e
if [ -x /usr/bin/update-desktop-database ]; then
    update-desktop-database -q /usr/share/applications || true
fi
if [ -x /usr/bin/gtk-update-icon-cache ]; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
EOF
chmod 755 "${PKG_DIR}/DEBIAN/postinst"

cat << 'EOF' > "${PKG_DIR}/DEBIAN/postrm"
#!/bin/sh
set -e
if [ -x /usr/bin/update-desktop-database ]; then
    update-desktop-database -q /usr/share/applications || true
fi
if [ -x /usr/bin/gtk-update-icon-cache ]; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
EOF
chmod 755 "${PKG_DIR}/DEBIAN/postrm"

# 7. 打包为 .deb
DIST_DIR="dist"
mkdir -p "$DIST_DIR"
DEB_FILE="${DIST_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"

echo "==> 正在使用 dpkg-deb 打包..."
dpkg-deb --build --root-owner-group "$PKG_DIR" "$DEB_FILE"

echo "=================================================="
echo " 打包成功! 生成产物:"
echo " 文件: ${DEB_FILE}"
echo " 大小: $(du -h "$DEB_FILE" | cut -f1)"
echo " SHA256: $(sha256sum "$DEB_FILE" | cut -d ' ' -f1)"
echo "=================================================="
echo ""
echo "本地安装测试命令:"
echo "  sudo dpkg -i ${DEB_FILE}"
echo ""
echo "卸载命令:"
echo "  sudo dpkg -r ${PACKAGE_NAME}"
echo "=================================================="
