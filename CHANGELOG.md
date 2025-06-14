<a name="v3.16.0"></a>
### v3.16.0 - 2025/06/14
- bacon colors configuration, see https://dystroy.org/bacon/config/#skin - Fix #215
- swift analyzers for building and linting - Thanks @mhedgpeth
- `open-jobs-menu` (bound to ctrl-j) and configurable `open-menu` - Fix #364

<a name="v3.15.0"></a>
### v3.15.0 - 2025/06/01
- the concept of "internals" has been removed. They were a category of actions and they're just actions now. This has no functional impact.
- the optional `workdir` job setting allows explicitely overriding the execution directory of a job's command - experimental, feedback welcome

<a name="v3.14.0"></a>
### v3.14.0 - 2025/05/19
- bacon no longer overrides `RUST_BACKTRACE` unless required (by env parameter or with `toggle_backtrace`) - Fix #356
- `toggle_backtrace` now accepts `0` as parameter. This allows defining a shortcut to disable externally set backtraces
- fix crash when search is active, output is wrapped and refreshed - Fix #359
- improve consistency of 'paused' state, which now also applies in case of config change

<a name="v3.13.0"></a>
### v3.13.0 - 2025/05/10
- add support for nextest 0.9.95 - Fix #350
- new `focus-file` internal. Example: `focus-file(some-partial-path)`
#### Remote Control
With `listen = true` in configuration, bacon listens for actions in a unix socket on file `.bacon.socket` in the package directory.
Goal is to allow controlling bacon from your code editor.
A simple way to try it is to launch `socat - UNIX-CONNECT:bacon.socket` then issue actions ending in new lines.
You may also use bacon with  `--send`, eg `bacon --send "job:clippy"`.

<a name="v3.12.0"></a>
### v3.12.0 - 2025/03/14
- cargo-json analyzer (for bacon-ls): fix invalid spans for errors from proc-macros - Fix #332 - Thanks @Leandros
- `back` action, usually mapped to the `esc` key, no longer quits on first job. If you want the old behavior, add this binding: `esc = "back-or-quit"`. Fix #338 - Thanks @ian-h-chamberlain

<a name="v3.11.0"></a>
### v3.11.0 - 2025/03/03
- hit `:` then type an integer to go to a diagnostic by number - Fix #104
- standard test analyzer: fix stack overflow not detected - Fix #326 - Thanks @gmorenz
#### Dynamic Completion
The script providing location aware completion needs to be sourced once.
This can be done for example in your .profile with
```bash
    source <(COMPLETE=bash bacon)
```
(adapt for your shell) This feature is still experimental. Please give feedback, positive or negative, in the chat.
Thanks @bryceberger

<a name="v3.10.0"></a>
### v3.10.0 - 2025/02/09
- all job parameters can now be specified at the root (to be applied to all jobs)
- `grace_period`, `show_change_count`, `sound.enabled`, and `sound.base_volume` can now be specified at job level
- `no-op` (no operation) internal, which can be used to disable a previously set binding

<a name="v3.9.1"></a>
### v3.9.1 - 2025/01/28
- as compilation of Alsa can be a problem on some systems, the "sound" feature is now disabled by default. You can enable it by compiling with `--features "sound"` - Fix #319

<a name="v3.9.0"></a>
### v3.9.0 - 2025/01/26
- sound can be enabled with `sound.enabled = true`, which allows adding job parameters such as `on_success = "play-sound(name=90s-game-ui-6,volume=50)"` and `on_failure = "play-sound(name=beep-warning)"` - Fix #303
- fix freeze on mac on config change - Fix #306 - Thanks @irh
- fix race condition on config reload when saved by helix - Fix #310

<a name="v3.8.0"></a>
### v3.8.0 - 2025/01/17
- with `--headless`, bacon runs without TUI - Fix #293
- `--config-toml` argument - Fix #284
- fix workspace level Cargo.toml file not watched
- `copy-unstyled-output` internal that you can bind with eg `ctrl-c = "copy-unstyled-output"`. It's currently gated by the `"clipboard"` feature, please give feedback regarding compilation and usage - Fix #282 - Thanks @letsgetrusty
- list of loaded config files displayed in help page

<a name="v3.7.0"></a>
### v3.7.0 - 2024/12/27
- search with the <kbd>/</kbd> key - Fix #224
- fix nextest analyzer not capturing test output with nextest 0.9.86+ - Fix #280
- show an error if the command fails to spawn - Thanks @jyn514

<a name="v3.6.0"></a>
### v3.6.0 - 2024/12/15
- support for cpp (gcc & clang) with `analyzer = "cpp"` - Thanks @bryceberger
- removal of the `--path` argument, replaced with `--project` and `--watch` (overrides the list of watched files). The path to the project can also be given as trailing argument as today. - Fix #274
- the `cargo_json` analyzer can now be leveraged to export data from the cargo metadata `Diagnostic` and `DiagnosticSpan` structs - Fix #249

<a name="v3.5.0"></a>
### v3.5.0 - 2024/12/05
- support for biome with `analyzer = "biome"`
- support for ruff with `analyzer = "python_ruff"`
- read bacon.toml in workspace/.config and package/.config - Fix #268
- read `workspace.metadata.bacon` and `package.metadata.bacon` config elements in `Cargo.toml` files - Fix #241
- fix locations export when launching bacon inside a rust workspace but with a non cargo tool

<a name="v3.4.0"></a>
### v3.4.0 - 2024/11/30
- new analyzer framework, make it possible for bacon to call more tools
- Python Pytest analyzer
- Analyzer for `cargo check --message-format json-diagnostic-rendered-ansi` - see #269
- allow specifying scroll-pages action with a floating point number - Fix #264

<a name="v3.3.0"></a>
### v3.3.0 - 2024/11/16
- bacon can now be launched without Cargo.toml file
- eslint analyzer (set `analyzer = "eslint"` in your job definition)
- Python Unittest analyzer (set `analyzer = "python_unittest"` in your job definition)
- fix Miri output seen as wrong when there's only warnings
- allow defining environment vars for all jobs - Thanks @joshka
- set `env.CARGO_TERM_COLOR = "always"` in default conf, thus making `"--color", "always"` useless in all cargo based job definition - Thanks @joshka
- new `ignore` job parameter, accepts a list of glob patterns
- more lenient detection of warnings and errors due to 'miri run' not supporting `--color` - Fix #251

<a name="v3.2.0"></a>
### v3.2.0 - 2024/11/04
- allow defining `default_watch` and `watch` at global level, so that they apply to all jobs unless overridden.

<a name="v3.1.2"></a>
### v3.1.2 - 2024/10/29
- "config loaded" message always automatically disappears after a few seconds

<a name="v3.1.1"></a>
### v3.1.1 - 2024/10/18
#### Major feature: hot reload of config files
When a configuration file is modified, bacon automatically reloads its config. So you don't need to quit/relaunch when you add a new job, add a key-binding, change the allowed lints of clippy, etc. - Fix #29

<a name="v3.0.0"></a>
### v3.0.0 - 2024/10/09
#### Major feature: nextest support
Hit `n` to launch the nextest job.

It's a default job, but you may define your own one by specifying `analyzer = "nextest"` in the job entry.

Internally, this is supported by a new analyzer framework which will allow easier analysis updates or addition of analysis for other tools (or languages).

Fix #196
#### Major feature: scope test job to failure
If you're running a test or nextest job and you want only the failing test to be retried, hit `f`.

If you want all tests to be executed again, hit `esc`.

Fix #214
#### Other features:
- grace period (by default 5ms) after a file event before the real launch of the command and during which other file events may be disregarded. Helps when saving a file changes several ones (eg backup then rename).
- new `exports` structure in configuration. New `analysis` export bound by default to `ctrl-e`. The old syntax defining locations export is still supported but won't appear in documentations anymore.
- recognize panic location in test - Fix #208
- lines to ignore can be specified as a set of regular expressions in a `ignored_lines` field either in the job or at the top of the prefs or bacon.toml - Fix #223
- `toggle-backtrace` accepts an optional level: `toggle-backtrace(1)` or `toggle-backtrace(full)` - Experimental - Fix #210
- configuration paths can be passed in `BACON_PREFS` and `BACON_CONFIG` env vars - Fix #76
#### Fixes:
- fix changing wrapping mode not always working in raw output mode - Fix #234

<a name="v2.21.0"></a>
### v2.21.0 - 2024/09/14
With `show_changes_count=true`, you can see the number of file changes that occurred since last job start.
#### Major change: the `on_change_strategy` setting and a new default strategy
* With `on_change_strategy = "kill_then_restart"`, the current job is immediately killed and a new job restarted. This is the behavior that bacon had before this PR. It has the downside of never allowing any job to complete if you're always changing files and the job is just a little too long to finish between changes.
* With `on_change_strategy = "wait_then_restart"` (which is the new default, so you can omit it), bacon waits for the job to finish before restarting it. This is probably much better when the jobs aren't instant and you want to continue changing files while it's computing.

The on_change_strategy can be defined in the global prefs, in the project settings, and even for a specific job.

<a name="v2.20.0"></a>
### v2.20.0 - 2024/08/13
- until now, when there was no `bacon.toml` file, the default one was applied, overriding the settings of `prefs.toml`. This is no longer the case: this default file is now applied before `prefs.toml` (which overrides it) - Fix #157
- `kill` job parameter - Thanks @pcapriotti

<a name="v2.19.0"></a>
### v2.19.0 - 2024/08/07
- `extraneous_args` job parameter - Thanks @TheTollingBell
- pause/unpause bound to 'p' - Fix #194

<a name="v2.18.2"></a>
### v2.18.2 - 2024/05/31
- fix failure to recognize location in test compilation output - Fix #190

<a name="v2.18.1"></a>
### v2.18.1 - 2024/05/21
- update dependencies (especially locked ones) to fix compilation on nightly

<a name="v2.18.0"></a>
### v2.18.0 - 2024/05/20
- new `{context}` possible part for exported locations, originally designed for [bacon-ls](https://github.com/crisidev/bacon-ls) but available for other purposes - Thanks @crisidev

<a name="v2.17.0"></a>
### v2.17.0 - 2024/05/05
- default binding for 'c' in bacon.toml is now the new 'clippy-all' job which does what the old 'clippy' job was doing. 'clippy' job changed to not run on all targets. Default bacon.toml explain how to bind 'c' to clippy instead of 'clippy-all' - Fix #167
- expand env vars in job command unless the job specifies `expand_env_vars = false` - Fix #181
- some file events filtered out from watch (feedback welcome, especially if you notice some failures to recompute)
- parse test results even when tests are run with `-q`/`--quiet` - Thanks @narpfel

<a name="v2.16.0"></a>
### v2.16.0 - 2024/03/30
- `on_success` triggered with warning or errors depending on `allow_warnings` and `allow_failures` - Fix #179
- `--no-help-line` option. This is experimental and may be removed depending on feedback and future additions to this line - Thanks @danielwolbach

<a name="v2.15.0"></a>
### v2.15.0 - 2024/03/05
- insert features related arguments before the -- when there's some - Fix #171
- fix offset in Windows terminal - Fix #175
- better `--help` with examples and main shortcuts
- rewriten execution engine

<a name="v2.14.2"></a>
### v2.14.2 - 2024/02/10
- update dependencies to fix bacon not compiling on nightly - Fix #168

<a name="v2.14.1"></a>
### v2.14.1 - 2023/12/24
- fix output not cleared when cargo is in quiet mode and there's nothing - Fix #131

<a name="v2.14.0"></a>
### v2.14.0 - 2023/10/04
- F5 now clears the output before running the job (now bound to `refresh` internal)
- new optional `background` job parameter, should be set to `false` for never ending jobs - Fix #161

<a name="v2.13.0"></a>
### v2.13.0 - 2023/09/15
- fix mouse wheel scrolling not working on Windows - Fix #153 - Thanks @Adham-A
- detect locations in test failures, thus enabling jumps to those failures
- more relevant suggestions in help line
- add a default job for running examples

<a name="v2.12.1"></a>
### v2.12.1 - 2023/07/22
- fix some scroll problem, especially in reverse - Fix #86

<a name="v2.12.0"></a>
### v2.12.0 - 2023/07/20
- better `--help`

<a name="v2.11.1"></a>
### v2.11.1 - 2023/07/13
- fix warning summary sometimes confused with a warning - Fix #149

<a name="v2.11.0"></a>
### v2.11.0 - 2023/06/30
- allow defining env vars for jobs - Fix #145

<a name="v2.10.0"></a>
### v2.10.0 - 2023/06/27
- accept bacon.toml file at workspace level - Fix #141

<a name="v2.9.0"></a>
### v2.9.0 - 2023/06/19
- export format and path can now be configured
- default export format includes error/warning summary (nvim-bacon has been updated in parallel) - Fix #127
- fix output non scrollable when non parsable
- fix test non parsed when styled and sent to stdout instead of stderr - Fix #137

<a name="v2.8.1"></a>
### v2.8.1 - 2023/04/22
- color rendering of cargo test - Fix #124

<a name="v2.8.0"></a>
### v2.8.0 - 2023/03/23
- By default, "src", "tests", "benches", "examples" are now watched - Fix #119
- `default_watch` bool job parameter - Fix #92

<a name="v2.7.0"></a>
### v2.7.0 - 2023/03/14
- watch "examples" directory in default run job
- fix warnings not recognized on Windows - Fix #70 - Thanks @crillon

<a name="v2.6.3"></a>
### v2.6.3 - 2023/03/09
- remove keybindings from default bacon.toml - Fix #116

<a name="v2.6.2"></a>
### v2.6.2 - 2023/03/03
- more consistent "pass!" - Thanks @zolrath

<a name="v2.6.1"></a>
### v2.6.1 - 2023/02/22
- fix a dependency compilation problem - Fix #112

<a name="v2.6.0"></a>
### v2.6.0 - 2023/02/21
- change default value of 'wrap' setting to true
- `--offline` experimental launch argument, prevents bacon (but not jobs) from accessing the network. Downside is a potentially less relevant list of watched files and directories - Fix #110

<a name="v2.5.0"></a>
### v2.5.0 - 2023/01/19
- new `allow_failures` job parameter - Fix #99
- `rerun` internal bound by default to F5 - Fix #105

<a name="v2.4.0"></a>
### v2.4.0 - 2023/01/12
Major feature:
The global prefs.toml and the local bacon.toml file now have the same properties, the local bacon.toml overriding the global prefs.toml file. Among the consequences: you can have a list of default global jobs; you can set a different preferences (eg wrapping, summary, etc.) for a specific repository. The default configuration files and the recommended best practices are unchanged - Fix #101

<a name="v2.3.0"></a>
### v2.3.0 - 2022/12/30
- doesn't launch job when the modified file is excluded by gitignore rules - Fix #32

<a name="v2.2.8"></a>
### v2.2.8 - 2022/12/15
- remove double-dash from default run configuration - Fix #96

<a name="v2.2.7"></a>
### v2.2.7 - 2022/12/14
- capture output of "should panic" tests - Fix #95

<a name="v2.2.6"></a>
### v2.2.6 - 2022/12/08
- fix a compilation problem - Fix #94

<a name="v2.2.5"></a>
### v2.2.5 - 2022/10/08
- fix wrong scrollbar in several cases of wrapping

<a name="v2.2.4"></a>
### v2.2.4 - 2022/10/05
- fix inability to scroll to last line sometimes

<a name="v2.2.3"></a>
### v2.2.3 - 2022/09/17
- fix a compilation problem on Window - Thanks @Stargateur - Fix #87

<a name="v2.2.2"></a>
### v2.2.2 - 2022/08/28
- define a new `allow_warnings` job setting. When it's true, the job is considered successful even when there are warnings. This is default on the `run` job, which means the `cargo run` output is displayed even when there are warnings - Fix #81
- allow `cargo --prefs` to be ran from outside cargo projects - Fix #84

<a name="v2.2.1"></a>
### v2.2.1 - 2022/05/12
- update some dependencies

<a name="v2.2.0"></a>
### v2.2.0 - 2022/05/12
- Locations exported in .bacon-locations now made absolute so that IDE plugins don't have to know the package's root
- job cancelling now works on unresponsive jobs too. This is a quite heavy change as the current implementation involves bringing in async and it's not 100% clean but it solves a major problem, further improvements could be welcome - Fix #78 - Thanks @nolanderc
- you can refer to cargo aliases by prefixing jobs with `alias:`, either when setting up keybindings, defaults, or when launching bacon. Example: `bacon alias:q` to launch the cargo task aliased as `q` - Fix #77

<a name="v2.1.0"></a>
### v2.1.0 - 2022/03/26
Major feature:
The `export-locations` argument (shortened in `-e`) generates a `.bacon-locations` file which can be used by IDE plugins.
A plugin has been made for neovim: [nvim-bacon](https://github.com/Canop/nvim-bacon) and other ones would be welcome.

Minor changes:
- wrapping now applies to all outputs, even non interpreted ones like the output of `cargo run`.

<a name="v2.0.1"></a>
### v2.0.1 - 2022/02/18
- fix summary of warnings counted as warning

<a name="v2.0.0"></a>
### v2.0.0 - 2022/02/16
#### Major features:
- It's now possible to configure key bindings in the prefs.toml file. Those key bingings can trigger internal actions (scrolling, toggling, quitting) or jobs (for example you can launch `cargo test` on the `t` key. - Fix #52
#### Other changes:
- help page, listing all key-bindings
- a job is said to be *successful* when there's no error, test failure or warning. When a job is successful, its output is displayed by bacon. This makes it possible to have a `cargo run` job, for example.
- it's possible to define an *action* to run when a job is successful. For example you can launch a `cargo doc --open` job on a key, and have bacon switch to the previous job with the `on_success = "back` trigger so that you don't open a browser page on every change
- arguments given after `--` are given to the job - Fix #67
- there's a web documentation site now, you should have a look: https://dystroy.org/bacon

Minor changes:
- fix character being lost behind scrollbar on wrapping
- replaced argh with clap for launch arg parsing. The `--help` presentation is thus different. `bacon -h` now supported.

<a name="v1.2.5"></a>
### v1.2.5 - 2022/01/29
- fix missing output of "no_run" doctests - Fix #64
- restrict naming of jobs to [\w-]+ regex (you were unlikely to use other chars due to the TOML format anyway)

<a name="v1.2.4"></a>
### v1.2.4 - 2021/11/27
- fix inability to deal with some inter-member dependencies on Windows - Fix #59 - Thanks @jDomantas
- fix compilation broken due to change in anyhow 1.0.49 - Fix #63

<a name="v1.2.3"></a>
### v1.2.3 - 2021/11/15
- add the "clippy-all" default job - Thanks @rukai
- alpha sort the table outputted by `bacon --list-jobs` - Thanks @rukai

<a name="v1.2.2"></a>
### v1.2.2 - 2021/10/18
- solve a dependency build problem - Fix #55

<a name="v1.2.1"></a>
### v1.2.1 - 2021/10/03
- propose to toggle backtraces when suggestion is found in cargo's output

<a name="v1.1.8"></a>
### v1.1.8 - 2021/07/31
- move to more recent versions of some crates - Fix #51
- `bacon --list-jobs` (or `bacon -l`) lists all available jobs

<a name="v1.1.7"></a>
### v1.1.7 - 2021/07/11
- recognize doc test output - Fix #49
- display 4 spaces for tabs - Fix #50

<a name="v1.1.6"></a>
### v1.1.6 - 2021/06/22
- the default conf now contains a [doc] job
- `--all-features` launch option

<a name="v1.1.5"></a>
### v1.1.5 - 2021/02/27
- fix wrong version number in bacon.log

<a name="v1.1.4"></a>
### v1.1.4 - 2021/02/10
It's possible to define directories to watch in the bacon.toml config file. For example, by default the `test` job watches the `tests` directory if it exists - Thanks @SafariMonkey

<a name="v1.1.3"></a>
### v1.1.3 - 2021/01/29
* `check-all` target now checks all - Fix #27
* `--no-default-features` and `--features` - Fix #31

<a name="v1.1.2"></a>
### v1.1.2 - 2021/01/05
Revert standard job to ignore tests because compilation with them is too slow. A new default job is added.

<a name="v1.1.1"></a>
### v1.1.1 - 2021/01/03
Don't consider test fails as command fails (ie display the count of test fails in `bacon test` instead of command error)

<a name="v1.1.0"></a>
### v1.1.0 - 2020/12/26
If the job's command returns an error code and no error was read in the output, bacon now displays the output and the error code instead of letting the user think there's no error

<a name="v1.0.1"></a>
### v1.0.1 - 2020/11/21
* vim key bindings can be enabled in prefs
* default job is now `cargo check --tests` to check the code for tests compiles too (without running them)

<a name="v1.0.0"></a>
### v1.0.0 - 2020/11/19
* nothing new... so it's stable enough to be tagged 1.0

<a name="v0.6.0"></a>
### v0.6.0 - 2020/11/15
* `bacon test` shows test failures - Fix #3 - Note that you need to remove then rebuild your bacon.toml file to use this new job

<a name="v0.5.3"></a>
### v0.5.3 - 2020/11/14
* "reverse" option allows having the focus on bottom - Fix #19
* initial compilation autoscroll based on scroll position - Fix #22
* remove flickering

<a name="v0.5.2"></a>
### v0.5.2 - 2020/11/14
* fix bacon ending with an error when prefs file is missing

<a name="v0.5.1"></a>
### v0.5.1 - 2020/11/13
* `bacon --prefs` shows or creates a prefs file which can be changed to defined default display settings
(currently "summary" and "wrap")

<a name="v0.5.0"></a>
### v0.5.0 - 2020/11/12
* `bacon --init` creates a default `bacon.toml` file which can be customized to add jobs or change the standard ones
* bacon launch arguments changed to ease use of customized jobs

<a name="v0.4.3"></a>
### v0.4.3 - 2020/11/11
* fix report only taking the first package into account (for workspaces)

<a name="v0.4.2"></a>
### v0.4.2 - 2020/11/11
* fix some regressions in error and warning detection

<a name="v0.4.1"></a>
### v0.4.1 - 2020/11/10
* reduce useless redraws during computation

<a name="v0.4.0"></a>
### v0.4.0 - 2020/11/10
* make it possible to watch only part of the sources: the passed directory (or the current one), when not a package directory (i.e. not containing a Cargo.toml file), will be the one watched - Thanks @nikhilmitrax and @jyn514 for their help
* logo - Thanks @petervaro
* line wrapping (and rewrapping on resize)

<a name="v0.3.2"></a>
### v0.3.2 - 2020/11/08
* when quitting bacon, kill `cargo check` if running

<a name="v0.3.1"></a>
### v0.3.1 - 2020/11/06
* better scroll position after toggling summary mode or resizing
* space key now usable for page down

<a name="v0.3.0"></a>
### v0.3.0 - 2020/11/06
* keep lines with location in summary mode - Fix #11
* allow scrolling the report (arrow keys, page keys, home & end keys, mouse wheel) - Fix #6
* log file renamed to 'bacon.log' to avoid collisions
* initial execution is displayed raw before report computation - Fix #8
* initial execution can be interrupted, scrolled - Fix #12

<a name="v0.2.0"></a>
### v0.2.0 - 2020-10-01
* add the summary mode

<a name="v0.1.1"></a>
### v0.1.1 - 2020-09-29
* also watches Cargo.toml

<a name="v0.1.0"></a>
### v0.1.0 - 2020-09-29
Initial version
