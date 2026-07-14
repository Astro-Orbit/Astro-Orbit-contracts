# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-07-14

### Added

- Comprehensive README with Mermaid ecosystem architecture diagram
- Network configuration examples (`.env.example`)
- Cross-repository documentation links
- CHANGELOG for release tracking

### Changed

- Workspace version bumped to 0.2.0

## [0.1.0] - 2026-07-13

### Added

- Organization contract: create, transfer, update_metadata, archive, get
- Project contract: register, archive, get
- Deployment contract: record, get, count
- Permissions contract: grant_role, revoke_role, has_role, get_role
- Registry contract: index_org, index_project, index_deployment, lookup_*
- Shared library: auth helpers, error types, event publishing, data types
- Soroban SDK v27.0.0 integration
- 30 unit tests across 5 contracts
- CI/CD pipelines (lint, test, build WASM, security audit, release)
- Release profile optimized for WASM size
