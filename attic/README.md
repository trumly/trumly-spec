# attic

Code that is kept for future use but is not built, tested or shipped.

`serve` was the local HTTP server behind `trumly serve` (a folder over HTTP for a visual editor) and `trumly collector`. It is out of the workspace and the binary. To bring it back, move it to `crates/serve`, add it to `members` in the root `Cargo.toml`, and restore the `serve` feature and the two commands in `crates/cli`.
