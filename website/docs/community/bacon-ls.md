[Bacon-ls](https://github.com/crisidev/bacon-ls) is a [language server](https://en.wikipedia.org/wiki/Language_Server_Protocol) implementation specifically designed to work with Bacon projects,
 exposing [textDocument/diagnostic](https://microsoft.github.io/language-server-protocol/specification#textDocument_diagnostic) and [workspace/diagnostic](https://microsoft.github.io/language-server-protocol/specification#workspace_diagnostic) capabilities to editors.

![bacon-ls](../img/bacon-ls.png)

**NOTE: Bacon-ls requires Bacon 3.7+ to work properly.**

**NOTE: Bacon-ls is not part of Bacon, it's a third-party tool developed to work WITH Bacon.**

# Installation

## VSCode

First, install [Bacon](../../#installation).

The VSCode extension is available on both VSCE and OVSX:

* `VSCE` [https://marketplace.visualstudio.com/items?itemName=MatteoBigoi.bacon-ls-vscode](https://marketplace.visualstudio.com/items?itemName=MatteoBigoi.bacon-ls-vscode)
* `OVSX` [https://open-vsx.org/extension/MatteoBigoi/bacon-ls-vscode](https://open-vsx.org/extension/MatteoBigoi/bacon-ls-vscode)

## Mason.nvim

Both Bacon and Bacon-ls are installable via [mason.nvim](https://github.com/williamboman/mason.nvim):

```vim
:MasonInstall bacon bacon-ls
```

## Manual

First, install [Bacon](../../#installation) and Bacon-ls

```bash
❯❯❯ cargo install --locked bacon bacon-ls
❯❯❯ bacon --version
bacon 3.7.0  # make sure you have at least 3.7.0
❯❯❯ bacon-ls --version
0.5.0        # make sure you have at least 0.5.0
```

# Configuration

Configure Bacon export settings with Bacon-ls export format and proper span support in `~/.config/bacon/prefs.toml`:

```toml
[jobs.bacon-ls]
command = [ "cargo", "clippy", "--tests", "--all-targets", "--all-features", "--message-format", "json-diagnostic-rendered-ansi" ]
analyzer = "cargo_json"
need_stdout = true

[exports.cargo-json-spans]
auto = true
exporter = "analyzer"
line_format = "{diagnostic.level}:{span.file_name}:{span.line_start}:{span.line_end}:{span.column_start}:{span.column_end}:{diagnostic.message}"
path = ".bacon-locations"
```

**NOTE: Bacon MUST be running to generate the export locations with the Bacon-ls job: `bacon -j bacon-ls`.**

The language server can be configured using the appropriate LSP protocol and
supports the following values:

- `locationsFile` Bacon export filename (default: `.bacon-locations`).
- `updateOnSave` Try to update diagnostics every time the file is saved (default: true).
- `updateOnSaveWaitMillis` How many milliseconds to wait before updating diagnostics after a save (default: 1000).
- `updateOnChange` Try to update diagnostics every time the file changes (default: true).

## Neovim - LazyVim

Bacon-ls is already integrated with [LazyVim](https://lazyvim.org) with [PR #3112](https://github.com/LazyVim/LazyVim/pull/3212):

```lua
vim.g.lazyvim_rust_diagnostics = "bacon-ls"
```

## Neovim - Manual

NeoVim requires [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig/) to be configured
and [rust-analyzer](https://rust-analyzer.github.io/) diagnostics must be turned off for Bacon-ls
to properly function.

Bacon-ls is part of nvim-lspconfig from commit
[6d2ae9f](https://github.com/neovim/nvim-lspconfig/commit/6d2ae9fdc3111a6e8fd5db2467aca11737195a30)
and it can be configured like any other LSP server works best when
[vim.diagnostics.opts.update_in_insert](https://neovim.io/doc/user/diagnostic.html#vim.diagnostic.Opts)
is set to `true`.

```lua
require("lspconfig").bacon_ls.setup({
    init_options = {
        updateOnSave = true
        updateOnSaveWaitMillis = 1000
        updateOnChange = false
    }
})
```

For `rust-analyzer`, these 2 options must be turned off:

```lua
rust-analyzer.checkOnSave.enable = false
rust-analyzer.diagnostics.enable = false
```

## VSCode

The extension can be configured using the VSCode settings interface.

## Coc.nvim

```vim
call coc#config('languageserver', {
      \ 'bacon-ls': {
      \   'command': '~/.cargo/bin/bacon-ls',
      \   'filetypes': ['rust'],
      \   'rootPatterns': ['.git/', 'Cargo.lock', 'Cargo.toml'],
      \   'initializationOptions': {
      \     'updateOnSave': v:true, 
      \     'updateOnSaveWaitMillis': 1000,
      \     'updateOnChange': v:false
      \   },
      \   'settings': {}
      \ }
\ })
```

# Troubleshooting

Bacon-ls can produce a log file in the folder where its running by exporting the `RUST_LOG` variable in the shell:

## Vim / Neovim

```bash
❯❯❯ export RUST_LOG=debug
❯❯❯ nvim src/some-file.rs                 # or vim src/some-file.rs
# the variable can also be exported for the current command and not for the whole shell
❯❯❯ RUST_LOG=debug nvim src/some-file.rs  # or RUST_LOG=debug vim src/some-file.rs
❯❯❯ tail -F ./bacon-ls.log
```

## VSCode

Enable debug logging in the extension options.

```bash
❯❯❯ tail -F ./bacon-ls.log
```

# How does it work?

Bacon-ls reads the diagnostics location list generated
by [Bacon's export-locations](../../config/#exports)
and exposes them on STDIO over the LSP protocol to be consumed
by the client diagnostics.

It requires Bacon to be running alongside
to ensure regular updates of the export locations.

The LSP client reads them as response to `textDocument/diagnostic` and `workspace/diagnostic`.

# More info

Please visit [https://github.com/crisidev/bacon-ls](https://github.com/crisidev/bacon-ls) for more information.
