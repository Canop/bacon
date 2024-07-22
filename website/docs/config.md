
# Configuration Files

The behavior of bacon is defined by a global `prefs.toml` file and project specific `bacon.toml` files.

All configuration files are optional but you'll soon need specific jobs for your targets, examples, etc.

All accept the same properties (preferences, keybindings, jobs, etc.).
The properties of the global `prefs.toml` file are overriden by the workspace level `bacon.toml` file, then by the package level `bacon.toml` file.

When you modified those files and bacon evolved since, you may want to have a look at the current default ones and pick the changes you like:

* [Current default prefs.toml](https://raw.githubusercontent.com/Canop/bacon/main/defaults/default-prefs.toml)
* [Current default bacon.toml](https://raw.githubusercontent.com/Canop/bacon/main/defaults/default-bacon.toml)

## Global Preferences

`bacon --prefs` creates the preferences file if it doesn't exist and returns its path (which is system dependent).

You may run `$EDITOR $(bacon --prefs)` to edit it directly.

The default configuration file contains already possible entries that you may uncomment and modify.

## Project Settings

`bacon --init` creates the `bacon.toml` file if it doesn't exist.

This file usually contains project specific jobs and shortcuts and should be saved and shared using your version control system.

It's a good idea to put here the triggers for specific jobs.

The [default bacon.toml](https://raw.githubusercontent.com/Canop/bacon/main/defaults/default-bacon.toml) is used when you don't create a file.

# Configuration Properties

## summary, wrap, reverse

You can change the `summary`, `wrapping`, and `reverse` mode at launch (see `bacon --help`), in the application using keys, and you may set the initial values in this preferences file:

```TOML
# Uncomment and change the value (true/false) to
# specify whether bacon should start in summary mode
#
# summary = true

# Uncomment and change the value (true/false) to
# specify whether bacon should start with lines wrapped
#
# wrap = true

# In "reverse" mode, the focus is at the bottom, item
# order is reversed, and the status bar is on top
#
# reverse = true
```


## Key Bindings

This section lets you change the key combinations to use to trigger [actions](#actions).

For example:

```TOML
[keybindings]
h = "job:clippy"
shift-F9 = "toggle-backtrace"
ctrl-r = "toggle-raw-output"
```

Note that you may have keybindings for jobs which aren't defined in your project, this isn't an error, and it's convenient to help keep define your personal keybindings in one place.

Another example, if you want vim-like shortcuts:

```TOML
[keybindings]
esc = "back"
g = "scroll-to-top"
shift-g = "scroll-to-bottom"
k = "scroll-lines(-1)"
j = "scroll-lines(1)"
ctrl-u = "scroll-page(-1)"
ctrl-d = "scroll-page(1)"
```

Your operating system and console intercept many key combinations. If you want to know which one are available, and the key syntax to use, you may find [print_key](https://github.com/Canop/print_key) useful.

## Jobs

A job is a command which is ran by bacon in background, and whose result is analyzed and displayed on end.

It's defined by the following fields:

field | optional | meaning
:-|:-:|:-
command | no | the tokens making the command to execute (first one is the executable)
watch | yes | a list of directories that will be watched if the job is run on a package. `src`, `tests`, `examples`, and `benches` are implicitly included unles `default_watch` is set to false
default_watch | yes | whether to watch default directories (`src`, `tests`, `examples`, and `benches`). `true` by default. When it's set to `false`, only the directories in `watch` are watched (none if `watch` is empty or not supplied)
need_stdout | yes |whether we need to capture stdout too (stderr is always captured). Default is `false`
on_success | yes | the action to run when there's no error, warning or test failures
allow_warnings | yes | if `true`, the action is considered a success even when there are warnings. Default is `false` but the standard `run` job is configured with `allow_warnings=true`
allow_failures | yes | if `true`, the action is considered a success even when there are test failures. Default is `false`
apply_gitignore | yes | if `true` (which is default) the job isn't triggered when the modified file is excluded by gitignore rules
env | yes | a map of environment vars, for example `env.LOG_LEVEL="die"`
background | yes | compute in background and display only on end. Default is `true`
extraneous_args | yes | if `false`, the action is run "as is" from `bacon.toml`, eg: no `--all-features` or `--features` inclusion. Default is `true`.

Example:

```TOML
[jobs.exs]
command = ["cargo", "run", "--example", "simple", "--color", "always"]
need_stdout = true
```

Don't forget to include `--color always` in most jobs, because bacon uses style information to parse the output of cargo.

Beware of job references in `on_success`: you must avoid loops with 2 jobs calling themselves mutually, which would make bacon run all the time.

## Default Job

The default job is the one which is launched when you don't specify one in argument to the bacon command (ie `bacon test`).
It's also the one you can run with the `job:default` action.

## export locations

If you use neovim, you probably want to use the [nvim-bacon](https://github.com/Canop/nvim-bacon) plugin.

This plugin needs location export to be enabled in your configuration:

```TOML
[export]
enabled = true
path = ".bacon-locations"
line_format = "{kind} {path}:{line}:{column} {message}"
```

You may change the path or line format for other tools.

# Actions

Actions are launched

* on key presses, depending on key-binding
* when triggered by a job ending success

Actions are parsed from strings, for example `quit` (long form: `internal:quit`) is the action of quitting bacon and can be bound to a key.

An action is either an *internal*, based on a hardcoded behavior of bacon, or a *job reference*

## Internals

internal | default binding | meaning
:-|:-|:-
back | <kbd>Esc</kbd> | get back to the previous page or job
help | <kbd>h</kbd> or <kbd>?</kbd> | open the help page
quit | <kbd>q</kbd> or <kbd>ctrl</kbd><kbd>q</kbd> or <kbd>ctrl</kbd><kbd>c</kbd> | quit
refresh | <kbd>F5</kbd> | clear output then run current job again
rerun |  | run current job again
toggle-raw-output |  | display the untransformed command output
toggle-backtrace | <kbd>b</kbd> | enable rust backtrace (for example on test failing)
toggle-summary | <kbd>s</kbd> | display results as abstracts
toggle-wrap | <kbd>w</kbd> | toggle line wrapping
scroll-to-top | <kbd>Home</kbd> | scroll to top
scroll-to-bottom | <kbd>End</kbd> | scroll to bottom
scroll-lines(-1) | <kbd>↑</kbd> | move one line up
scroll-lines(1) | <kbd>↓</kbd> | move one line down
scroll-pages(-1) | <kbd>PageUp</kbd> | move one page up
scroll-pages(1) | <kbd>PageDown</kbd> | move one page down

The `scroll-lines` and `scroll-pages` internals are parameterized.
You can for example define a shortcut to move down 5 lines:

```toml
shift-d = "scroll-lines(5)"
```

## Job References

Job references are useful as actions, which can be bound to key combinations.

They're either role based or name based.

To refer to the job called `test`, you use a name based reference: `job:test`.

To refer to a job based on a [cargo alias](https://doc.rust-lang.org/cargo/reference/config.html#alias), add `alias:`, for example `job:alias:r`.

Role based job references are the following ones:

job reference | meaning
-|-
`job:default` | the job defined as *default* in the bacon.toml file
`job:initial` | the job specified as argument, or the default one if there was none explicit
`job:previous` | the job which ran before, if any (or we would quit). The `back` internal has usually the same effect
