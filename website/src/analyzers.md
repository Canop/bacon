While Bacon was initially developed for the Rust language, it covers more and more tools and language.

For most of them, a dedicated `analyzer` must be specified in the [job settings](../config#jobs).

This page is an overview of the supported tools and how bacon can be configured for them.

Some analyzers are flagged as **<span style="background-color:orange;color:white;padding:3px">young</span>**: it means some feedback would be welcome, either in the [chat](https://miaou.dystroy.org/4683) (if positive) or as GitHub issue (if negative).

# Summary

Analyzer | Languages | Tool
-|-|-
[standard](#rust) (*default*) | Rust | [cargo](https://doc.rust-lang.org/cargo/) `check`, `build`, `test`, `clippy`, `doc`, `run`, `miri`
[cargo_json](#cargojson)| Rust |  cargo with `--message-format json-diagnostic-rendered-ansi`
[nextest](#nextest)| Rust |  [cargo-nextest](https://nexte.st/)
[cpp](#gcc-clang)| C++ |  Clang and GCC
cpp_doctest| C++ |  [doctest](https://github.com/doctest/doctest).
[biome](#biome)| JS/TS/CSS |  [Biome](https://biomejs.dev/)
[eslint](#eslint)| JS/TS/CSS |  [ESLint](https://eslint.org/)
[go](#go-build) | Go | [go](https://go.dev/)
[python_pytest](#pytest)| Python |  [pytest](https://docs.pytest.org/)
[python_ruff](#ruff)| Python |  [ruff](https://docs.astral.sh/ruff/)
[python_unittest](#unittest) | Python |  [Unittest](https://docs.python.org/3/library/unittest.html)
[swift_build](#swift-build)| Swift |  swift build
[swift_lint](#swift-lint)| Swift |  swift lint
[typescript](#typescript)| TS | [tsc](https://www.typescriptlang.org/docs/handbook/compiler-options.html)

# Rust

Rust specific support of bacon includes reading Cargo.toml files to identify all source directories, and help managing cargo features.

## Cargo build, clippy, test, doc, run

**Status: <span style="background-color:green;color:white;padding:3px">mature</span>**

These tools produce warnings, errors, test failures, with the same representation.

Bacon comes with preconfigured modifiable jobs for them, and you can add some with no explicit analyzer according to your specific needs, for example

```TOML
[jobs.nightly-clippy]
command = [
	"cargo",
	"+nightly",
	"clippy",
	"--all-targets",
	"--",
	"-A", "clippy::bool_to_int_with_if",
]
```

## Cargo/JSON

**Status: <span style="background-color:green;color:white;padding:3px">mature</span>**

Cargo can be configured to output JSON.

```TOML
[jobs.json-check]
command = [
	"cargo", "check",
	"--message-format", "json-diagnostic-rendered-ansi",
]
need_stdout = true
analyzer = "cargo_json"
```

With the `cargo_json` analyzer, the visible result in bacon is the same, but using this analyzer makes it possible to export from bacon more detailed data to use in other tools, eg [bacon-ls](https://github.com/crisidev/bacon-ls).

## Miri

**Status: <span style="background-color:green;color:white;padding:3px">mature</span>**

[miri](https://github.com/rust-lang/miri) is supported with the default analyzer.

Bacon isn't preconfigured for miri but you can add a job with

```TOML
[jobs.miri]
command = ["cargo", "+nightly", "miri", "run"]
need_stdout = true
```

## Nextest

**Status: <span style="background-color:green;color:white;padding:3px">mature</span>**

[nextest](https://nexte.st/)

It doesn't use the standard analyzer but bacon comes preconfigured with a nextest job so that you can launch `bacon nextest` or simply hit <kbd>n</kbd> while in bacon.


# C++

## GCC / Clang

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

```TOML
[jobs.gcc]
command = [
    "g++", "-Wall", "src/main.cpp",
]
watch = ["src"]
need_stdout = true
analyzer = "cpp"
```

# Go

## Go build

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

Example configuration:

```TOML
command = [
    "go", "build",
    "-v",
    "./...",
]
need_stdout = true
analyzer = "go"
watch = ["."]
```

# Python

Support of Python is just starting, and Python developers should raise their hand if they want to see progress here.

## Pytest

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

[pytest](https://docs.pytest.org/en/stable/)

It's configured with

```TOML
[jobs.pytest]
command = [
    "pytest"
]
need_stdout = true
analyzer = "python_pytest"
watch = ["."]
```

## Ruff

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

[ruff](https://docs.astral.sh/ruff/)

Example configuration:

```TOML
[jobs.ruff]
env.FORCE_COLOR = "1"
command = [
    "ruff", "check",
]
need_stdout = true
analyzer = "python_ruff"
watch = ["."]
```

## Unittest

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

Support for the [Unittest](https://docs.python.org/3/library/unittest.html) framework seems to work, but lacks testers and users.

Example configuration:

```TOML
[jobs.unittest]
command = [
    "python3", "unitest_runner.py",
]
need_stdout = true
analyzer = "python_unittest"
watch = ["."]
```

# JavaScript / TypeScript

## Biome

**Status: <span style="background-color:green;color:white;padding:3px">mature</span>**

[Biome](https://biomejs.dev/)

Example configuration (for a `./libs` folder) with some lint rules skipped:

```TOML
[jobs.biome-libs]
env.RAYON_NUM_THREADS = "1" # for constant ordering of items
command = [
    "npx", "@biomejs/biome", "lint",
    "--colors", "force",
    "./libs",
    "--skip", "complexity/useArrowFunction",
    "--skip", "style/useTemplate",
]
need_stdout = true
analyzer = "biome"
watch = ["libs"]
```

## Eslint

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

[ESLint](https://eslint.org/)

```TOML
[jobs.lint]
command = ["npx", "eslint", "--color", "libs/*"]
need_stdout = true
analyzer = "eslint"
watch = ["libs"]
```

## TypeScript

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

Example configuration:

```TOML
command = [
    "tsc",
    "--noEmit", "--pretty",
]
analyzer = "typescript"
need_stdout = true
watch = ["src"]
```

# Swift

**Status: <span style="background-color:orange;color:white;padding:3px">young</span>**

[Swift](https://www.swift.org/)

## Swift build

Example configuration:

```TOML
[jobs.swift_build]
command = ["swift", "build"]
watch = ["Sources"]
need_stdout = true
analyzer = "swift_build"
```

## Swift lint

Example configuration:

```TOML
command = ["swiftlint", "lint", "--config", ".swiftlint.yml", "--strict"]
watch = ["Sources"]
need_stdout = true
analyzer = "swift_lint"
```

# Other tools

What's not here, you should probably ask for it, either on [GitHub](https://github.com/Canop/bacon) or on [the Miaou chat](https://miaou.dystroy.org/4683).
