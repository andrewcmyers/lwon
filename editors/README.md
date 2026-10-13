# Editor support for LWON

Syntax highlighting for `.lwon` files.

## VS Code

The extension in `vscode/` uses the TextMate grammar
`vscode/syntaxes/lwon.tmLanguage.json`. To use it without packaging it,
link it into the VS Code extensions directory and restart VS Code:

    ln -s "$PWD/vscode" ~/.vscode/extensions/lwon

Alternatively, package it with `npx @vscode/vsce package` in `vscode/`
and install the resulting `.vsix` file with "Extensions: Install from
VSIX...".

To test the grammar:

    cd vscode
    npx vscode-tmgrammar-test -g syntaxes/lwon.tmLanguage.json 'test/*.test.lwon'

Lines of `vscode/test/syntax.test.lwon` that begin with `#` and
contains `^` or `<-` assert the scopes of the characters above.

## Vim

The directory `vim/` is a Vim plugin providing filetype detection, syntax
highlighting, and comment settings. Install it as a package:

    mkdir -p ~/.vim/pack/lwon/start
    ln -s "$PWD/vim" ~/.vim/pack/lwon/start/lwon

or add it to `runtimepath` in your `.vimrc`, or load it with a plugin
manager. Neovim uses the same files; its package directory is
`~/.local/share/nvim/site/pack`.

To test the syntax highlighting, which checks the same sample file as the
VS Code test:

    cd vim
    vim -Nu NONE -es -c 'set rtp^=.' -c 'filetype plugin on' -c 'syntax on' -S test/run.vim

## GitHub (Linguist)

GitHub highlights files using Linguist, which uses TextMate grammars, so
the VS Code grammar can serve for LWON there too. Linguist adds a new
language only once its file extension is in use in many repositories on
GitHub; see Linguist's contribution guidelines.
