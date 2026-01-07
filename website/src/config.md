
# Configuration Files

All configuration files are optional but you'll probably need specific jobs for your targets, examples, etc.

All configuration files accept the same properties (preferences, keybindings, jobs, etc.).

Bacon loads in order:

* its default internal configuration (which includes the default bacon.toml)
* the global `prefs.toml` ([global preferences](#global-preferences))
* the file whose path is in environment variable `BACON_PREFS`
* the `workspace.metadata.bacon` entry in the workspace's `Cargo.toml`
* the workspace level `bacon.toml` file ([project settings](#project-settings))
* the `bacon.toml` file in `workspace-root/.config/`
* the `package.metadata.bacon` entry in the package's `Cargo.toml`
* the `bacon.toml` file in `package-root/`
* the `bacon.toml` file in `package-root/.config/`
* the file whose path is in environment variable `BACON_CONFIG`
* the content of the `--config-toml` argument

Each configuration file overrides the properties of previously loaded ones.

But you don't *need* so many files.
It's usually enough to have a global `prefs.toml` file and a project specific `bacon.toml`.

When you modified those files and bacon evolved since, you may want to have a look at the current default ones:

* [Current default prefs.toml](https://raw.githubusercontent.com/Canop/bacon/main/defaults/default-prefs.toml)
* [Current default bacon.toml](https://raw.githubusercontent.com/Canop/bacon/main/defaults/default-bacon.toml)

Bacon watches those files and reload them when necessary, so you don't have to relaunch it if you add a key-binding, or a job, or [an allowed lint](../cookbook/#configure-clippy-lints) in your clippy job.

## Global Preferences

`bacon --prefs` creates the preferences file if it doesn't exist and returns its path (which is system dependent).

You may run `$EDITOR "$(bacon --prefs)"` to edit it directly.

The default configuration file contains already possible entries that you may uncomment and modify.

## Project Settings

`bacon --init` creates the `bacon.toml` file if it doesn't exist.

This file usually contains project specific jobs and shortcuts and should be saved and shared using your version control system.

It's a good idea to put here the triggers for specific jobs.


# Jobs

A job is a command which is ran by bacon in background, and whose result is analyzed and displayed on end.

There's always an active job in bacon, be it the default one or one you selected at launch or using a bound key.

A job declaration in a TOML file looks like this:

```TOML
[jobs.clippy-all]
command = [
	"cargo", "clippy",
	"--",
	"-A", "clippy::derive_partial_eq_without_eq",
	"-A", "clippy::len_without_is_empty",
	"-A", "clippy::map_entry",
]
need_stdout = false
```

## Job Properties

The job is defined by the following fields:

field | meaning | default
:-|:-|:-
allow_failures | if `true`, the action is considered a success even when there are test failures | `false`
allow_warnings | if `true`, the action is considered a success even when there are warnings | `false`
analyzer | command output parser, see below | `"standard"`
apply_gitignore | if `true` the job isn't triggered when the modified file is excluded by gitignore rules | `true`
background | compute in background and display only on end | `true`
command | the tokens making the command to execute (first one is the executable) |
default_watch | whether to watch default files (`src`, `tests`, `examples`, `build.rs`, and `benches`). When it's set to `false`, only the files in your `watch` parameter are watched | `true`
env | a map of environment vars, for example `env.LOG_LEVEL="die"` |
hide_scrollbar | whether to hide the scrollbar (for easier select & copy) | `false`
kill | a command replacing the default job interruption (platform dependant, `SIGKILL` on unix). For example `kill = ["kill", "-s", "INT"]` |
ignore | list of glob patterns for files to ignore |
ignored_lines | regular expressions for lines to ignore |
extraneous_args | if `false`, the action is run "as is" from `bacon.toml`, eg: no `--all-features` or `--features` inclusion | `true`
need_stdout |whether we need to capture stdout too (stderr is always captured) | `false`
on_change_strategy | `wait_then_restart` or `kill_then_restart` |
on_success | the action to run when there's no error, warning or test failures |
skin | bacon application colors, [see below](#skin) |
watch | a list of files and directories that will be watched if the job is run on a package. Usual source directories are implicitly included unless `default_watch` is set to false |
workdir | overrides the execution directory |

All these properties can also be defined before jobs and will apply to all of them unless overridden.

Beware of job references in `on_success`: you must avoid loops with 2 jobs calling themselves mutually, which would make bacon run all the time.

Example:

```TOML
[jobs.exs]
command = ["cargo", "run", "--example", "simple"]
need_stdout = true
```

Note: Some tools detect that their output is piped and don't add style information unless you add a parameter which usually looks like `--color always`.
This isn't normally necessary for cargo because bacon, by default, sets the `CARGO_TERM_COLOR` environment variable.

## Analyzers

The output of the standard cargo tools is understood by bacon's standard analyzer.

For other tools, a specific analyzer may be configured with, eg, `analyzer = "nextest"`.

For the list of analyzers and configuration examples, see [Analyzers](../analyzers).

## Default Job

The default job is the one which is launched when you don't specify one in argument to the bacon command (ie `bacon test`).
It's also the one you can run with the `job:default` action.

You can set the default job by setting the `default_job` key in your `bacon.toml` file.


# Key Bindings

This section lets you change the key combinations to use to trigger [actions](#actions).

For example:

```TOML
[keybindings]
h = "job:clippy"
shift-F9 = "toggle-backtrace(1)"
ctrl-r = "toggle-raw-output"
cmd-e = "play-sound"
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

# Actions

Actions are launched

* on key presses, depending on key-binding
* when triggered by a job ending success

Actions are parsed from strings, for example `quit` is the action of quitting bacon and can be bound to a key.

action | default binding | meaning
:-|:-|:-
back | <kbd>Esc</kbd> | get back to the previous page or job, or cancel search
back-or-quit | | back to previous page or job, quitting if there is none
copy-unstyled-output | | write the currently displayed job output to the clipboard
dismiss-top-item | <kbd>alt</kbd><kbd>i</kbd> | [dismiss](../cookbook/#deal-with-pedantic) the top item
dismiss-top-item-type | | dismiss the top item's type
dismiss-top | <kbd>alt</kbd><kbd>t</kbd> | dismiss the top item's (its type if possible)
export:name | | runs the [export](#exports) with this name
focus-file(path) | | show this file's diagnostics first
focus-search | <kbd>/</kbd> | focus the search input
help | <kbd>h</kbd> or <kbd>?</kbd> | open the help page
job:job-reference | | execute the job with [job-reference](#job-references)
next-match | <kbd>tab</kbd> | go to next search match
no-op |  | do nothing (may be used to disable a previously set binding)
open-menu(*menu-definition*) |  | open a user defined menu. For example<br>` "open-menu(intro=a text,actions=[job:ch,export:mx,quit])"`
open-undismiss-menu | <kbd>alt</kbd><kbd>t</kbd> | open a menu to [undismiss](../cookbook/#deal-with-pedantic) chosen items
open-jobs-menu | <kbd>ctrl</kbd>-<kbd>j</kbd> | open a menu with all jobs
pause |  | disable automatic job execution on change
play-sound |  | play a [sound](#sound) with optional parameters, eg `play-sound(volume=100%)`
previous-match | <kbd>backtab</kbd> | go to previous search match
quit | <kbd>q</kbd> or <kbd>ctrl</kbd><kbd>q</kbd> or <kbd>ctrl</kbd><kbd>c</kbd> | quit
refresh | <kbd>F5</kbd> | clear output then run current job again
reload-config | | reload all configuration files
rerun |  | run current job again
scope-to-failures | <kbd>f</kbd> | restrict job to test failure(s)
scroll-lines(-1) | <kbd>↑</kbd> | move one line up
scroll-lines(1) | <kbd>↓</kbd> | move one line down
scroll-pages(-1) | <kbd>PageUp</kbd> | move one page up
scroll-pages(1) | <kbd>PageDown</kbd> | move one page down
scroll-to-bottom | <kbd>End</kbd> | scroll to bottom
scroll-to-top | <kbd>Home</kbd> | scroll to top
toggle pause | <kbd>p</kbd> | toggle pause
toggle-backtrace(level) | <kbd>b</kbd> | enable rust backtrace, level is either `0`, `1`, `2`, or `full`
toggle-raw-output |  | display the untransformed command output
toggle-summary | <kbd>s</kbd> | display results as abstracts
toggle-wrap | <kbd>w</kbd> | toggle line wrapping
undismiss-all |  | [undismiss](../cookbook/#deal-with-pedantic) all items
unpause |  | enable automatic job execution on change
validate | <kbd>enter</kbd> | unfocus the input, keeping the search

The `scroll-lines` and `scroll-pages` actions are parameterized.
You can for example define a shortcut to move down half a page:

```toml
ctrl-d = "scroll-pages(.5)"
```

# Job References

Job references are useful as actions, which can be bound to key combinations.

They're either role based or name based.

To refer to the job called `test`, you use a name based reference: `job:test`.

To refer to a job based on a [cargo alias](https://doc.rust-lang.org/cargo/reference/config.html#alias), add `alias:`, for example `job:alias:r`.

Role based job references are the following ones:

job reference | meaning
-|-
`job:default` | the job defined as *default* in the bacon.toml file
`job:initial` | the job specified as argument, or the default one if there was none explicit
`job:previous` | the job which ran before, if any. The `back` action has usually the same effect
`job:previous-or-quit` | same as `job:previous`, but will quit if there was no job. The `back-or-quit` action has usually the same effect

# Exports

If necessary, exports can be defined to write files either on end of task or on key presses.

Following are 3 typical configurations.

## Locations export

This is the best way to ensure you can list warnings/errors/failures and navigate between them in your IDE, for example with a plugin such as [nvim-bacon](https://github.com/Canop/nvim-bacon).

With the following configuration, locations are exported on each job execution.

```TOML
[exports.locations]
auto = true
path = ".bacon-locations"
line_format = "{kind} {path}:{line}:{column} {message}"
```

This export works for any tool and any job.

### Available template variables

The following variables can be used in `line_format`:

variable | meaning
:-|:-
`{kind}` | the diagnostic kind: `error`, `warning`, or `test`
`{job}` | the job name (e.g., `check`, `clippy`, `test`)
`{item_idx}` | the diagnostic item index (1, 2, ...)
`{path}` | absolute file path
`{line}` | line number
`{column}` | column number (defaults to 1 if not available)
`{message}` | the diagnostic message
`{context}` | the full diagnostic context (all lines for the item)

Example with all variables:

```TOML
[exports.locations]
auto = true
path = ".bacon-locations"
line_format = "{kind}@{job}[{item_idx}] {path}:{line}:{column} {message}"
```

This would produce output like:
```
error@clippy[1] /path/to/file.rs:42:5 mismatched types
warning@clippy[2] /path/to/file.rs:10:1 unused variable
```

## Cargo Spans export

When using the `cargo_json` analyzer, more detailed information is available than what is printed on screen and this analyzer can provide that data with a configuration such as this one:

```TOML
[jobs.bacon-ls]
command = [ "cargo", "clippy", "--message-format", "json-diagnostic-rendered-ansi" ]
analyzer = "cargo_json"
need_stdout = true

[exports.cargo-json-spans]
auto = true
exporter = "analyzer"
line_format = "{diagnostic.level}:{span.file_name}:{span.line_start}:{span.line_end}:{diagnostic.message}"
```

The exported data come from the [Diagnostic](https://docs.rs/cargo_metadata/0.19.1/cargo_metadata/diagnostic/struct.Diagnostic.html) and [DiagnosticSpan](https://docs.rs/cargo_metadata/0.19.1/cargo_metadata/diagnostic/struct.DiagnosticSpan.html) structures.

## Report export

This is an example of exporting a report on a key press, when required:


```TOML
[exports.json-report]
auto = false

[keybindings]
ctrl-e = "export:json-report"
```

# Other config properties

Have a look, at least once, at the default configuration files.
They contain many other properties, commented out, that you may find useful.

## listen

Setting `listen = true` in the configuration makes bacon listen for commands on a `.bacon.socket` unix socket, in the package directory (i.e. next to the `.bacon-locations` file).

Any action that can be bound to a key can also be sent on this socket, in text, one action per line.

You can use standard unix programs. For example run `socat - UNIX-CONNECT:bacon.socket` then issue actions ending in new lines.

Bacon can also be used to send those actions, eg `bacon --send 'scroll-lines(-2)'`.

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

## Sound

You may have audio notifications on job success or failures.

This requires sound to be enabled, either at root level or in a specific job:

```TOML
[sound]
enabled = true
base_volume = "100%" # global volume multiplier
```

Sound being enabled, you can add `play-sound` callbacks to jobs, eg

```TOML
on_success = "play-sound(name=90s-game-ui-6,volume=50)"
on_failure = "play-sound(name=beep-warning,volume=100)"
```

Sound name can be omitted. Possible values are `2`, `90s-game-ui-6`, `beep-6`, `beep-beep`, `beep-warning`, `bell-chord`, `car-horn`, `convenience-store-ring`, `cow-bells`, `pickup`, `positive-beeps`, `short-beep-tone`, `slash`, `store-scanner`, `success`.

## Skin

Most colors of the bacon application can be redefined in a `skin`, with colors being [8 bit ANSI values](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit).

You can set colors within a `[skin]` object in any configuration file:

```TOML
[skin]
status_fg = 251
status_bg = 4
key_fg = 11
status_key_fg = 11
project_name_badge_fg = 11
project_name_badge_bg = 69
```
and you can override colors in a job:

```TOML
[jobs.test]
command = ["cargo", "test"]
need_stdout = true
skin.status_bg = 6
```

All available skin entries, with meaning and default values, are listed in [src/conf/skin.rs](https://github.com/Canop/bacon/blob/main/src/conf/skin.rs#62).
Skin entries may be added or removed in minor versions of bacon. Unrecognized entries are just ignored.

