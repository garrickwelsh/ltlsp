FROM mcr.microsoft.com/devcontainers/base:noble as intermediate

ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

WORKDIR /tmp/build

RUN set -x \
  && apt-get update \
  && apt-get install -y software-properties-common \
  && add-apt-repository ppa:maveonair/helix-editor \
  && add-apt-repository ppa:neovim-ppa/unstable \
  && apt-get update \
  && apt-get install -y helix tmux tealdeer ripgrep neovim \
  # Rust specific
      rustup lldb \ 
  && locale-gen en_AU.UTF-8 && echo "LANG=en_AU.UTF-8" > /etc/locale.conf \
  # Install just
  && JUST_VERSION=$(curl -s "https://api.github.com/repos/casey/just/releases/latest" | jq -r .tag_name) \ 
  && pkgname=just \
  && curl -Lo just.tar.gz "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
  && tar zxvf just.tar.gz \
  && rm just.tar.gz \
  && install -Dm755 -t "${pkgdir}/usr/local/bin/" "${pkgname}" \
  && install -Dm644 -t "${pkgdir}/usr/local/share/man/man1/" "${pkgname}.1" \
  && install -Dm644 -t "${pkgdir}/usr/local/share/licenses/${pkgname}/" "LICENSE" \
  && install -Dm644 -t "${pkgdir}/usr/local/share/bash-completion/completions/" "completions/${pkgname}.bash" \
  && install -Dm644 -t "${pkgdir}/usr/local/share/elvish/lib/" "completions/${pkgname}.elvish" \
  && install -Dm644 -t "${pkgdir}/usr/local/share/fish/vendor_completions.d/" "completions/${pkgname}.fish" \
  && install -Dm644 -t "${pkgdir}/usr/local/share/zsh/site-functions/" "completions/${pkgname}.zsh" \
  # Install difftastic
  && DIFFT_VERSION=$(curl -s "https://api.github.com/repos/Wilfred/difftastic/releases/latest" | \grep -Po '"tag_name": *"\K[^"]*') \ 
  && pkgname=difft \
  && curl -Lo difft-x86_64-unknown-linux-gnu.tar.gz https://github.com/Wilfred/difftastic/releases/download/${DIFFT_VERSION}/difft-x86_64-unknown-linux-gnu.tar.gz \
  && tar zxvf difft-x86_64-unknown-linux-gnu.tar.gz \
  && rm difft-x86_64-unknown-linux-gnu.tar.gz \
  && install -Dm755 -t "/usr/local/bin" "${pkgname}" \
  # Install omnisharp
  && OMNISHARP_VERSION=$(curl -s "https://api.github.com/repos/OmniSharp/omnisharp-roslyn/releases/latest" | \grep -Po '"tag_name": *"v\K[^"]*') \ 
  && curl -Lo omnisharp-linux-x64-net6.0.tar.gz https://github.com/OmniSharp/omnisharp-roslyn/releases/download/v${OMNISHARP_VERSION}/omnisharp-linux-x64-net6.0.tar.gz \
  && tar zxvf omnisharp-linux-x64-net6.0.tar.gz \
  && pkgname="OmniSharp" \
  && install -Dm755 OmniSharp *.dll -t"$pkgdir"/usr/lib/omnisharp \
  && install -Dm644 *.json          -t"$pkgdir"/usr/lib/omnisharp \
  && install -dm755                   "$pkgdir"/usr/bin \
  && ln -sf /usr/lib/omnisharp/OmniSharp "$pkgdir"/usr/bin/omnisharp \
  && ln -sf /usr/lib/omnisharp/OmniSharp "$pkgdir"/usr/bin/OmniSharp \
  && install -Dm644 license.md        "$pkgdir"/usr/share/licenses/$pkgname/license.md \
  # lunarvim pre-requistes
  && apt-get install -y python3-pip npm \ 
  # clean up
  && apt-get clean \
  && rm -r *

RUN set -x \
    && npm install --global tree-sitter-cli \
    # Install Git UI
    && GITUI_VERSION=$(curl -s "https://api.github.com/repos/extrawurst/gitui/releases/latest" | jq -r .tag_name) \
    && curl -Lo gitui-linux-x86_64.tar.gz "https://github.com/extrawurst/gitui/releases/download/${GITUI_VERSION}/gitui-linux-x86_64.tar.gz" \
    && tar xvf gitui-linux-x86_64.tar.gz \
    && rm gitui-linux-x86_64.tar.gz \
    && install -Dm755 -t "/usr/local/bin" gitui \
    # Install marksman
    && MARKSMAN_VERSION=$(curl -s "https://api.github.com/repos/artempyanykh/marksman/releases/latest" | \grep -Po '"tag_name": *"\K[^"]*') \
    && curl -Lo marksman https://github.com/artempyanykh/marksman/releases/download/$MARKSMAN_VERSION/marksman-linux-x64 \
    && install -Dm755 -t "/usr/local/bin" marksman \
    # Install markdown-oxide
    && MARKDOWN_OXIDE_VERSION=$(curl -s "https://api.github.com/repos/Feel-ix-343/markdown-oxide/releases/latest" | \grep -Po '"tag_name": *"v\K[^"]*') \
    && curl -Lo markdown-oxide https://github.com/Feel-ix-343/markdown-oxide/releases/download/v$MARKDOWN_OXIDE_VERSION/markdown-oxide-v${MARKDOWN_OXIDE_VERSION}-x86_64-unknown-linux-gnu \
    && install -Dm755 -t "/usr/local/bin" markdown-oxide \
    && rm markdown-oxide \
    && apt-get install -y asciinema \
    && LAZYGIT_VERSION=$(curl -s "https://api.github.com/repos/jesseduffield/lazygit/releases/latest" | \grep -Po '"tag_name": *"v\K[^"]*') \
    && curl -Lo lazygit.tar.gz "https://github.com/jesseduffield/lazygit/releases/download/v${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION}_Linux_x86_64.tar.gz" \
    && tar zxf lazygit.tar.gz lazygit \
    && install -Dm755 -t "/usr/local/bin" lazygit \
    && rm lazygit.tar.gz \
    && echo "Done"

USER $USERNAME

RUN set -x \
     echo "Install rust tools" \
     && rustup toolchain install stable \
     && rustup toolchain install nightly  \
     && rustup default stable  \
     && rustup component add rust-analyzer rust-docs rust-src clippy cargo rustfmt \
     && echo "Finished installing rust tools"

RUN set -x  \ 
      # Install lunarvim (tree-sitter-cli is required to install lunarvim)
      && LV_BRANCH='release-1.4/neovim-0.9' \
      && curl -s https://raw.githubusercontent.com/LunarVim/LunarVim/${LV_BRANCH}/utils/installer/install.sh|bash \
      # Set difftastic as diff util
      && git config --global diff.external difft \ 
      # Fix for tmux not displaying unicode characters.
      && echo "export LANG=en_AU.UTF-8" >> ~/.bashrc

WORKDIR /workspaces

######### Code Snippets ###########

  # Setup vscode user if they don't already exist
  # && groupadd --gid $USER_GID $USERNAME \
  # && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
  # && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
  # && chmod 0440 /etc/sudoers.d/$USERNAME \
