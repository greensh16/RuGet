# Changelog

## [0.2.0] - YYYY-MM-DD
### Added
- Introduced deprecated aliases for `RuGetError`, `ErrorCode`, and `ErrorKind` to maintain backward compatibility without breaking downstream crates. Use the new types directly, as these aliases will be removed in version 1.0.0.
- Deprecated methods `legacy_new`, `legacy_io_error`, and `legacy_http_error` inside `RuGetError`. Use the recommended methods (`new`, `from`) directly instead.

### Breaking Changes
- None yet, but deprecated aliases and methods will be removed in version 1.0.0.

### Migration
- Replace any usage of `RuGetErrorLegacy`, `ErrorCodeLegacy`, and `ErrorKindLegacy` with `RuGetError`, `ErrorCode`, and `ErrorKind` respectively.
- Update calls from any deprecated `legacy_*` methods to the new methods recommended in the deprecation notes.

### Semantic Versioning Guidance
- Minor version bump to 0.2.0 to accommodate non-breaking deprecation additions,
- Plan for a major version bump to 1.0.0 to remove deprecated items after downstream crates have had time to migrate.

Refer to individual deprecation notes within the code for guidance on how to migrate.
