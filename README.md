
**bacon** is a **bac**kground **com**piler.

It's designed for minimal interaction so that you can jut let it running, side to your editor, and be notified of warnings and errors in your Rust code.

## FAQ

### What does it do ?

It watches the content of your src directory and launches `cargo check` on changes.

Watching and computations are done on background threads to prevent any blocking.

The screen isn't cleaned until the compilation is finished to prevent flickering.

The rendering is adapted to the dimensions of the terminal to ensure you get a proper usable report.

Errors are priviledged: by default warnings are only displayed when there's no error.

### Does it work on Windows?

Not yet.

### Why "bacon" ?

It comes from France and, as you know, France is bacon.
