# Filesystem Conventions

<!--
This is a minimal template. Feel free to add more sections as needed.

Please review also the Design Doc template and add any relevant sections to your ADR:
https://www.notion.so/pl-strflt/Writing-a-Design-Doc-aa6034be43c2434ba88a2fd844516e94
-->

> Status: ACCEPTED

<!--
PROPOSED, ACCEPTED, REJECTED, DEPRECATED, SUPERSEDED BY {link-to-ADR}
-->

## Context

Station runtime and modules need to store various kinds of data. Some of the data is temporary (e.g.
content cached by dCDN nodes), while other is expected to be persistent (e.g. statistics like the
number of jobs completed). Persistent data should be included in backups (e.g. the stats), while
temporary data must be excluded from backups.

The current configuration with a single `ROOT_DIR` directory is not sufficient to support these
different requirements. See the discussion in
[filecoin-station/core#75](https://github.com/filecoin-station/core/issues/75) for more details.

## Options Considered

1. Treat all data as temporary and store it in the cache directory (e.g. `XDG_CACHE_HOME`). The
   problem with this solution is that when the OS purges cache directories, we will lose our state
   and report an incorrect number of completed jobs.

2. Treat all data as persistent and store it in the state directory (e.g. `XDG_STATE_HOME`). This
   may unnecessarily increase backup sizes, e.g. for macOS Time Machine users.

3. Replace single `ROOT_DIR` with several fine-grained locations. This adds a bit of complexity to
   the configuration, especially regarding computing sensible defaults in Station Core, Station
   Desktop, Zinnia Daemon and Zinnia CLI. On the other hand, using different locations for different
   kinds of data is a common practice used by all operating systems (Windows, macOS, Unix, and
   freedesktop/XDG to name a few).

## Decision

Remove `ROOT_DIR` configuration option and introduce two new configurable locations:

- `CACHE_ROOT` for storing temporary files. This directory must not be shared with other computers
  (e.g. via Windows roaming) and should be excluded from backups by the default OS convention.

- `STATE_ROOT` for storing persistent files. This directory must not be shared with other computers
  (e.g. via Windows roaming) and should be included in backups by default.

In the future, we may introduce additional locations.

<!--
What is the change that we're proposing and/or doing?
-->

## Consequences

Zinnia runtime and untrusted modules will get fine-grained control of the way how the host OS treats
their files.

Filecoin Station will use computer resources efficiently, keeping backup sizes reasonably small and
supporting OS-specific features like the Windows feature to delete temporary files across all apps.

## Links &amp; References

- [Discussion on GitHub](https://github.com/filecoin-station/core/issues/75)
- [Apple File System Programming Guide](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html)
- [macOS Library Directory Details](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/MacOSXDirectories/MacOSXDirectories.html#//apple_ref/doc/uid/TP40010672-CH10-SW1)
- [XDG Base Directory specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [Windows Environment Variables](https://ss64.com/nt/syntax-variables.html)
