# Bacon Configuration Examples

This directory contains example configuration files demonstrating various features of Bacon.

## Environment Variable Examples

### `example.env`
A sample environment file showing the format supported by the `env_file` configuration option:
- Comments (lines starting with `#`)
- KEY=VALUE pairs
- Quoted values (both single and double quotes)
- Empty values

### `example-bacon.toml`
An example `bacon.toml` configuration file demonstrating:
- Global `env_file` usage
- Job-specific `env_file` configuration
- How direct `env` variables override `env_file` variables

### Test Files

#### `test.env`
A test environment file used for testing the `env_file` feature.

#### `test-bacon.toml`
A test configuration file for validating the `env_file` functionality.

## Usage

To use these examples in your project:

1. Copy the relevant files to your project root
2. Modify the values to match your project's needs
3. Update the `env_file` paths in your `bacon.toml` if needed

For more information about Bacon configuration, see the [official documentation](https://dystroy.org/bacon/config/).
