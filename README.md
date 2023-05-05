![bacon][logo]

[logo]: img/logo-text.png?raw=true "bacon"

[![Latest Version][s1]][l1] [![Chat on Miaou][s2]][l2]

[s1]: https://img.shields.io/crates/v/bacon.svg
[l1]: https://crates.io/crates/bacon

[s2]: https://miaou.dystroy.org/static/shields/room.svg
[l2]: https://miaou.dystroy.org/3768?rust


**bacon** is a background rust code checker.

It's designed for minimal interaction so that you can just let it run, alongside your editor, and be notified of warnings, errors, or test failures in your Rust code.

![screenshot](doc/screenshot.png)

# Documentation

The **[bacon website](https://dystroy.org/bacon)** is a complete guide.

Below is a short overview.

## install

    cargo install --locked bacon

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

![bacon test](doc/test.png)

## define your own jobs

First create a `bacon.toml` file by running

    bacon --init

This file already contains some standard jobs. Add your own, for example

```toml
[jobs.check-win]
command = ["cargo", "check", "--target", "x86_64-pc-windows-gnu", "--color", "always"]
```

or

```toml
[jobs.check-examples]
command = ["cargo", "check", "--examples", "--color", "always"]
watch = ["examples"] # src is implicitly included
```

*Don't forget the `--color always` part: bacon uses style information to recognize warnings and errors.*

and run

    bacon check-win

or

    bacon check-examples

The `bacon.toml` file may evolve with the features and settings of your project and should be added to source control.


## Licences

Bacon is licenced under [AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html).
You're free to use it to compile the Rust projects of your choice, even commercial.

The logo is designed by [Peter Varo][pv] and licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License][cc-lic].
[![license][cc-img]][cc-lic]

[pv]: https://petervaro.com
[cc-lic]: https://creativecommons.org/licenses/by-sa/4.0
[cc-img]: https://i.creativecommons.org/l/by-sa/4.0/80x15.png
