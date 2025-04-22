pkgname=tmux-leap
pkgver=1.8.0
pkgrel=1
pkgdesc="tmux leaper, fzf through a list of projects or directories, autosessionizing, history"
arch=('x86_64' 'aarch64')
url="https://github.com/fibsussy/tmux-leap"
license=('MIT')
depends=('fzf' 'tmux')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/fibsussy/tmux-leap/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')
options=('!debug')  # Explicitly disable debug package generation

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

    # Install the binary
    install -Dm755 "target/release/tmux-leap" "$pkgdir/usr/bin/tmux-leap"

    # Dynamically generate completion scripts
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions"

    ./target/release/tmux-leap completion bash > "$pkgdir/usr/share/bash-completion/completions/tmux-leap"
    ./target/release/tmux-leap completion zsh > "$pkgdir/usr/share/zsh/site-functions/_tmux-leap"
}
