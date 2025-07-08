pkgname=tmux-leap
pkgver=1.8.3
pkgrel=1
pkgdesc="tmux leaper, fzf through a list of projects or directories, autosessionizing, history"
arch=('x86_64' 'aarch64')
url="https://github.com/fibsussy/tmux-leap"
license=('MIT')
depends=('fzf' 'tmux')
makedepends=('curl')
source=("https://github.com/fibsussy/tmux-leap/releases/download/v${pkgver}/tmux-leap-linux-${CARCH}.tar.gz"
        "https://raw.githubusercontent.com/fibsussy/tmux-leap/main/LICENSE")
sha256sums=('SKIP'
            'SKIP')
options=('!debug')

package() {
    install -Dm755 "tmux-leap" "$pkgdir/usr/bin/tmux-leap"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 <(./tmux-leap completion bash) "$pkgdir/usr/share/bash-completion/completions/tmux-leap"
    install -Dm644 <(./tmux-leap completion zsh) "$pkgdir/usr/share/zsh/site-functions/_tmux-leap"
}
