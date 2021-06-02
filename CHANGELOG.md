### next
- the default conf now contains a [doc] job

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
