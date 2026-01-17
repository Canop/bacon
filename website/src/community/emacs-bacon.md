

# Purpose

[emacs-bacon](https://seed.pipapo.org/nodes/seed.pipapo.org/rad:zKbD2Y9kERBYScgczMaJBTRfjBhh)
is an emacs minor-mode, which, combined with bacon, lets you navigate between errors and
warnings without leaving your editor, just hitting a key. It also syncronizes bacon to show
the current navigated item.

# Development

emacs-bacon is maintained on [radicle](https://radicle.xyz/).
It's radicle id is `rad:zKbD2Y9kERBYScgczMaJBTRfjBhh`.
Issues and PR's are maintained in [radicle](https://radicle.xyz/guides/user#2-collaborating-the-radicle-way) as well.

Discussions and feedback are welcome on mastodon. Write to `@cehteh@social.tchncs.de` with the
`#scrollpanel` tag.


# How it works

At every job end, bacon writes a `.bacon-locations` file with all items (errors, warnings,
test failures, etc.) and for each of them its label, file path, line and column.

It read the `bacon/prefs.toml` and local `bacon.toml`. You can remote-control bacon from emacs
with the `C-c b` prefix (customizeable) followed by whatever keybindings the bacon preferences
define to send a command to bacon.

You can bind `bacon-next` and other commands to jump to next|prev|first
items|error|warning|test. bacon-minor-mode watches the `.bacon-locations` file for changes and
re-reads it every time it changed.

When jumping to a location, emacs-bacon also sends a command to bacon to scroll to the focused item.

# Bacon configuration

The configuration instructing bacon to export the locations at every job should be defined in
your global `bacon/prefs.toml`:

```TOML
[exports.locations]
auto = true
path = ".bacon-locations"
line_format = "{kind}@{job}[{item_idx}] {path}:{line}:{column} {message}"
```

In order for emacs-bacon to send commands to bacon, you should also ask it to listen, with

```toml
listen = true
```

# Installation & Usage

More details on how to install emacs-bacon, how to configure it, and how to use it, are
described in its own page:
[emacs-bacon](https://seed.pipapo.org/nodes/seed.pipapo.org/rad:zKbD2Y9kERBYScgczMaJBTRfjBhh)
