# Maintainer: Ruan <ruanvasconcelos898@gmail.com>
pkgname=easy-remote
pkgver=0.1.0
pkgrel=1
pkgdesc="Open-source 'Single Click' remote support client for Linux."
arch=('x86_64')
url="https://github.com/RuanVasco/easy-remote"
license=('GPL3')
depends=('gtk4' 'libadwaita' 'glib2' 'x11vnc' 'wayvnc')
makedepends=('cargo' 'git')
source=("git+https://github.com/RuanVasco/easy-remote.git#branch=main")
md5sums=('SKIP')

backup=('etc/easy-remote/client/entries.xml') 

options=('!lto')

prepare() {
    cd "easy-remote"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "easy-remote"
    cargo build --release --frozen --all-features -p easy-client
}

package() {
    cd "easy-remote"

    install -Dm755 "target/release/easy-client" "$pkgdir/usr/bin/easy-remote-client"

    install -Dm644 "crates/easy-client/assets/com.github.RuanVasco.easy-client.desktop" \
        "$pkgdir/usr/share/applications/com.github.RuanVasco.easy-client.desktop"

    install -Dm644 "crates/easy-client/assets/com.github.RuanVasco.easy-client.svg" \
        "$pkgdir/usr/share/icons/hicolor/scalable/apps/com.github.RuanVasco.easy-client.svg"

    install -Dm644 "crates/easy-client/assets/entries.xml" \
        "$pkgdir/etc/easy-remote/client/entries.xml"
}