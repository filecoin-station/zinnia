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

1. Wait for the
   [Release CI/CD Workflow](https://github.com/filecoin-station/zinnia/actions/workflows/release.yml)
   to finish. This usually takes about 25-30 minutes.

1. Find the Draft Release created by the Release workflow in
   [releases](https://github.com/filecoin-station/zinnia/releases)

1. Click on the button `Generate release notes`. Review the list of commits and pick a few notable
   ones. Add a new section `Highlights ✨` at the top of the release notes and describe the selected
   changes.

1. Click on the green button `Publish release`
