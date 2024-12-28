pkgname=jumper
pkgver=1.1.0
pkgrel=2
pkgdesc="tmux jumper, fzf through a list of projects"
arch=('x86_64' 'aarch64')
url="https://github.com/fibalious/jumper"
license=('MIT')
depends=('fzf')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/fibalious/jumper/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')
options=('!debug')  # Explicitly disable debug package generation

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

    # Install the binary
    install -Dm755 "target/release/jumper" "$pkgdir/usr/bin/jumper"

    # Dynamically generate completion scripts
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions"

    ./target/release/jumper completion bash > "$pkgdir/usr/share/bash-completion/completions/jumper"
    ./target/release/jumper completion zsh > "$pkgdir/usr/share/zsh/site-functions/_jumper"
}
