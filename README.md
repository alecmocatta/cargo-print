# cargo-print

[![Crates.io](https://img.shields.io/crates/v/cargo-print.svg?maxAge=86400)](https://crates.io/crates/cargo-print)
[![MIT / Apache 2.0 licensed](https://img.shields.io/crates/l/cargo-print.svg?maxAge=2592000)](#License)
[![Build Status](https://dev.azure.com/alecmocatta/cargo-print/_apis/build/status/tests?branchName=master)](https://dev.azure.com/alecmocatta/cargo-print/_build?definitionId=22)

[📖 Docs](https://docs.rs/cargo-print) | [💬 Chat](https://constellation.zulipchat.com/#narrow/stream/213236-subprojects)

A cargo subcommand to print information in a shell-convenient format. Useful for CI.

Can be installed and run like so:

```text
cargo install cargo-print
cargo print examples [--no-default-features] [--features <FEATURES>...] [--all-features]
# prints examples that can be run given the specified features
cargo print publish
# prints packages of a workspace in order for publishing
cargo print package
# prints the name of the package in the current directory
cargo print directory <package-name>
# prints the directory of the specified package
```

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
