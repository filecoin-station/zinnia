# Release process

This project follows [semantic versioning](https://semver.org/). The following
documentation will refer to `X.Y.Z` as _major_, _minor_ and _patch_ version.

## Releasing one or more crates

### Prerequisites

- [cargo release](https://github.com/crate-ci/cargo-release/)

### Steps

1. Make sure you have the latest version of the `main` branch

   ```sh
   $ git checkout main && git pull
   ```

2. Create the new release

   ```sh
   $ cargo release --workspace --sign-tag --no-push --execute
   ```
