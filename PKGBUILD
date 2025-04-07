pkgname=leap
pkgver=1.5.1
pkgrel=1
pkgdesc="tmux leaper, fzf through a list of projects"
arch=('x86_64' 'aarch64')
url="https://github.com/fibsussy/leap"
license=('MIT')
depends=('fzf' 'tmux')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/fibsussy/leap/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')
options=('!debug')  # Explicitly disable debug package generation

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

    # Install the binary
    install -Dm755 "target/release/leap" "$pkgdir/usr/bin/leap"

    # Dynamically generate completion scripts
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions"

    ./target/release/leap completion bash > "$pkgdir/usr/share/bash-completion/completions/leap"
    ./target/release/leap completion zsh > "$pkgdir/usr/share/zsh/site-functions/_leap"
}
