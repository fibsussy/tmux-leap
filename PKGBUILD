
pkgname=jumper
pkgver=0.1.1
pkgrel=0
pkgdesc="fzf through a list of projects"
arch=('x86_64')
url="https://github.com/yourusername/jumper"
license=('MIT')
depends=('fzf')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/yourusername/jumper/archive/v$pkgver.tar.gz")
sha256sums=('SKIP') # Replace SKIP with the actual checksum later

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 "target/release/jumper" "$pkgdir/usr/bin/jumper"

    install -Dm644 "completions/jumper.bash" "$pkgdir/usr/share/bash-completion/completions/jumper"
    install -Dm644 "completions/_jumper" "$pkgdir/usr/share/zsh/site-functions/_jumper"
}
