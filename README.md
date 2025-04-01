![bacon][logo]

[logo]: img/logo-text.png?raw=true "bacon"

[![Latest Version][s1]][l1] [![site][s4]][l4] [![Chat on Miaou][s2]][l2] [![License: AGPL v3][s3]][l3]

[s1]: https://img.shields.io/crates/v/bacon.svg
[l1]: https://crates.io/crates/bacon

[s2]: https://dystroy.org/chat-shield.svg
[l2]: https://miaou.dystroy.org/4683?bacon

[s3]: https://img.shields.io/badge/License-AGPL_v3-blue.svg
[l3]: https://www.gnu.org/licenses/agpl-3.0

[s4]: https://dystroy.org/dystroy-doc-pink-shield.svg
[l4]: https://dystroy.org/bacon

**bacon** is a background code checker.

It's designed for minimal interaction so that you can just let it run, alongside your editor, and be notified of warnings, errors, or test failures in your Rust code.

![screenshot](doc/screenshot.png)

# Documentation

The **[bacon website](https://dystroy.org/bacon)** is a complete guide.

Below is only a short overview.

## install

    cargo install --locked bacon

Run this command too if you want to update bacon. Configuration has always been retro-compatible so you won't lose anything.

Some features are disabled by default. You may enable them with

    cargo install --features "clipboard sound default-sounds"

## check the current project

    bacon

That's how you'll usually launch bacon, because other jobs like `test`, `clippy`, `doc`, your own ones, are just a key away: You'll hit <kbd>c</kbd> to see Clippy warnings, <kbd>t</kbd> for the tests, <kbd>d</kbd> to open the documentation, etc.


## check another project

    bacon --path ../broot

or

    bacon ../broot

## check all targets (tests, examples, benches, etc)

    bacon --job check-all

When there's no ambiguity, you may omit the `--job` part:

    bacon check-all

## run clippy instead of cargo check

    bacon clippy

This will run against all targets like `check-all` does.

## run tests

    bacon test

or `bacon nextest` if you're a nextest user.

![bacon test](doc/test.png)


When there's a failure, hit <kbd>f</kbd> to restrict the job to the failing test.
Hit <kbd>esc</kbd> to get back to all tests.

## define your own jobs

First create a `bacon.toml` file by running

    bacon --init

This file already contains some standard jobs. Add your own, for example

```toml
[jobs.check-win]
command = ["cargo", "check", "--target", "x86_64-pc-windows-gnu"]
```

or

```toml
[jobs.check-examples]
command = ["cargo", "check", "--examples"]
watch = ["examples"] # src is implicitly included
```

and run

    bacon check-win

or

    bacon check-examples

The `bacon.toml` file may evolve with the features and settings of your project and should be added to source control.

## Optional features

Some bacon features can be disabled or enabled at compilation:

* `"clipboard"` - disabled by default : necessary for the `copy-unstyled-output` internal
* `"sound"` - disabled by default : necessary for the `play-sound` internal
* `"default-sounds"` - disabled by default: embed some default sounds for the `play-sound` internal

## Licences

Bacon is licenced under [AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html).
You're free to use it to compile the Rust projects of your choice, even commercial.

The logo is designed by [Peter Varo][pv] and licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License][cc-lic].
[![license][cc-img]][cc-lic]

[pv]: https://petervaro.com
[cc-lic]: https://creativecommons.org/licenses/by-sa/4.0
[cc-img]: https://i.creativecommons.org/l/by-sa/4.0/80x15.png
