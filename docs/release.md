# Release process

This project follows [semantic versioning](https://semver.org/). The following documentation will
refer to `X.Y.Z` as _major_, _minor_ and _patch_ version.

## Releasing one or more crates

### Prerequisites

- [cargo release](https://github.com/crate-ci/cargo-release/)

### Steps

1. Make sure you have the latest version of the `main` branch:

   ```sh
   $ git checkout main && git pull
   ```

1. Create the new release, replace `0.1.2` with the NEW version number:

   ```sh
   $ cargo release --workspace --sign-tag --tag-prefix "" --execute 0.1.2
   ```

1. Continue on GitHub to create a new Release:

   1. Open https://github.com/filecoin-station/zinnia/releases/new
   1. Pick the tag created in the step above, e.g. `zinnia-v0.0.2`
   1. Click on the button `Generate release notes`
   1. Click on the green button `Publish release`
