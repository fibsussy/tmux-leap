pkgname=jumper
pkgver=0.1.1
pkgrel=2
pkgdesc="fzf through a list of projects"
arch=('x86_64')
url="https://github.com/fibalious/jumper"
license=('MIT')
depends=('fzf')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/fibalious/jumper/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

    # Install the binary
    install -Dm755 "target/release/jumper" "$pkgdir/usr/bin/jumper"

    # Dynamically generate completion scripts
    install -Dm644 /dev/null "$pkgdir/usr/share/bash-completion/completions/jumper"
    install -Dm644 /dev/null "$pkgdir/usr/share/zsh/site-functions/_jumper"

    ./target/release/jumper completion bash > "$pkgdir/usr/share/bash-completion/completions/jumper"
    ./target/release/jumper completion zsh > "$pkgdir/usr/share/zsh/site-functions/_jumper"
}
