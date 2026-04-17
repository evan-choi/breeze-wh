# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/evan-choi/breeze-wh/compare/v0.1.3...v0.1.4) - 2026-04-17

### Added

- add `breeze-wh upgrade` command

### Other

- Merge remote-tracking branch 'origin/main' into dev

## [0.1.3](https://github.com/evan-choi/breeze-wh/compare/v0.1.2...v0.1.3) - 2026-04-17

### Other

- Merge pull request #6 from evan-choi/dev
- aggressive size optimization in release profile

## [0.1.2](https://github.com/evan-choi/breeze-wh/compare/v0.1.1...v0.1.2) - 2026-04-17

### Fixed

- upload release exe as breeze-wh.exe (no version suffix)

## [0.1.1](https://github.com/evan-choi/breeze-wh/compare/v0.1.0...v0.1.1) - 2026-04-16

### Fixed

- correct release-plz action path

### Other

- Switch to release-plz for automated releases
- Release workflow: publish before build, upload raw exe only
- Bump version to 0.1.1
- Release workflow: gracefully skip cargo publish if version exists
- Simplify README to install/uninstall only
- Auto-start service on install
- Update README for crates.io install and Breeze-WH paths
- Add cargo publish to release workflow
