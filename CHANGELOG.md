# CHANGELOG.md

## 0.3.0 (2026-03-23)

Features:
  - Implemented large dataset streaming support with chunked input processing, overlap handling, and optional aggregation pass -> [#84](https://github.com/brianbirrell/ai-cli/pull/84)
  - Added configurable input processing mode (`off`, `chunked`, `auto`) and chunking settings in config -> [#84](https://github.com/brianbirrell/ai-cli/pull/84)
  - Added terminal activity spinner with `--no-progress` support and TTY-safe behavior -> [#83](https://github.com/brianbirrell/ai-cli/pull/83)
  - Added automated release workflow (`Automate Release`) with SemVer version validation and tag management -> [#84](https://github.com/brianbirrell/ai-cli/pull/84)
  - Added automated Linux release packaging (`tar.gz`, `zip`, and `deb`) in CI workflows -> [#81](https://github.com/brianbirrell/ai-cli/pull/81)

Security and CI:
  - Added explicit workflow permissions to address code scanning alerts -> [#61](https://github.com/brianbirrell/ai-cli/pull/61), [#62](https://github.com/brianbirrell/ai-cli/pull/62), [#63](https://github.com/brianbirrell/ai-cli/pull/63)
  - Updated deprecated GitHub Actions usage to current compatible actions and cache versions -> [#82](https://github.com/brianbirrell/ai-cli/pull/82)
  - Enhanced automate-release workflow with additional validation checks and release flow improvements

Dependencies:
  - Removed `fs` dependency and updated lockfile/dependency set as part of modernization work -> [#79](https://github.com/brianbirrell/ai-cli/pull/79)
  - Updated dependencies including `clap`, `serde`, `libc`, `mockito`, `log`, and `rustls-webpki`

Documentation:
  - Added large dataset streaming specification -> [docs/specs/LARGE_DATASET_STREAMING_SPEC.md](docs/specs/LARGE_DATASET_STREAMING_SPEC.md)
  - Updated release automation guidance in README.md
  - Updated version management guidance for the release process

## 0.2.0 (2025-09-01)

Features:
  - Added LLM temperature control option -> [#12](https://github.com/brianbirrell/ai-cli/issues/12) and [#54](https://github.com/brianbirrell/ai-cli/issues/54)
  - Added connection timeout configuration option -> [#12](https://github.com/brianbirrell/ai-cli/issues/12)
  - Enhance logging of service calls -> [#14](https://github.com/brianbirrell/ai-cli/issues/14)
  - Update input data prompt text -> [#16](https://github.com/brianbirrell/ai-cli/issues/16)
  - Add the OpenAI AGENTS.md specification to the project -> [#52](https://github.com/brianbirrell/ai-cli/issues/52)

Dependencies:
  - Updated multiple dependencies to latest versions

## 0.1.2 (2025-08-08)

Security:
  - None for this release

Features:
  - Updated input data prompt text -> [#16](https://github.com/brianbirrell/ai-cli/issues/16)
  - Added version information command line arg -> [#5](https://github.com/brianbirrell/ai-cli/issues/5)
  - Added version information command line arg -> [#5](https://github.com/brianbirrell/ai-cli/issues/5)
  - Added verbose output command line arg -> [#4](https://github.com/brianbirrell/ai-cli/issues/4)

Fix:
  - Fixed authentication issue the API key from config file were ignored -> [#8](https://github.com/brianbirrell/ai-cli/issues/8)

## 0.1.0 (2025-07-31)

Features:
  - Initial working code -> [02fc6b4](https://github.com/brianbirrell/ai-cli/commit/02fc6b4115a04db74973ba972fb06d0e61ae9161)
