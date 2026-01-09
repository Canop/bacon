

# Purpose

[nvim-bacon](https://github.com/Canop/nvim-bacon) is a neovim plugin, which, combined with bacon, lets you navigate between errors and warnings without leaving your editor, just hitting a key.

# How it works

At every job end, bacon writes a `.bacon-locations` file with all items (errors, warnings, test failures, etc.) and for each of them its label, file path, line and column.

In nvim, on predefined shortcuts, the nvim-bacon plugin may jump to the next item's position, or display all items to let you choose one. The plugin reads the `.bacon-locations` file every time you hit one of its shortcuts.

When jumping to a location, nvim also sends a command to bacon to scroll to the focused item.

Nothing in this design is Rust related.
This plugin can thus be used whatever the ecosystem(s) you program in.

# Bacon configuration

The configuration instructing bacon to export the locations at every job should be defined in your global `bacon/prefs.toml`:

```TOML
[exports.locations]
auto = true
path = ".bacon-locations"
line_format = "{kind} {path}:{line}:{column} {message}"
```

In order for nvim-bacon to send commands to bacon, you should also ask it to listen, with

```toml
listen = true
```

# Installation & Usage

How to install nvim-bacon, how to configure it, and how to use it, are described in its own page: [https://github.com/Canop/nvim-bacon](https://github.com/Canop/nvim-bacon).
