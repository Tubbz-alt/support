# Contributor Guidelines

- Before implementing a feature, discuss the feature by creating an issue for it.
- Before creating a commit, run `cargo +nightly fmt` on the project.
- Commits should follow the [Conventional Commit] guidelines

## Tips

- `rustup` is the preferred tool for managing Rust toolchains as a developer
- `cargo clippy` can point out the majority common mistakes in Rust code
- `sbuild` can be used to verify that debian packages build correctly
- `unsafe` is explicitly disallowed, unless otherwise permitted by a maintainer

[conventional commit]: https://www.conventionalcommits.org/en/v1.0.0-beta.4/
