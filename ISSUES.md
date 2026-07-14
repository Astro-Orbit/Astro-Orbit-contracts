# Astro Orbit — Engineering Issues

> Generated from full-source audit

## Contents

1. [Ship Blockers](#1-ship-blockers)
2. [Architecture & Design](#2-contracts-architecture--design)
3. [Security & Auth](#3-contracts-security--auth)
4. [Testing Quality](#4-contracts-testing-quality)
5. [Events & Storage](#5-contracts-events--storage)
6. [Documentation & Config](#6-contracts-documentation--config)
7. [CI/CD & Tooling](#7-contracts-cicd--tooling)
8. [Cross-Cutting Testing](#8-cross-cutting-testing)
9. [Developer Experience](#9-developer-experience)

---

# 1. Ship Blockers

---

## Issue #8 — Contract `init` functions have no authorization — anyone can front-run initialization

**Labels:** bug, contracts, blocker

**Summary:**
All 5 contracts' `init()` functions accept admin/owner parameters but never call `require_auth()`. On public testnets, any address that calls `init` first can claim admin/owner control. The `Owner` field in `Organization` contract and `admin` in all other contracts is set without authentication.

**Acceptance Criteria:**
- [ ] All 5 init functions call `env.invoker.require_auth()` (or equivalent) before storing config
- [ ] At minimum, the deployer must sign to initialize
- [ ] Tests verify `init` by unauthorized address is rejected

**Files:** `contracts/organization/src/lib.rs:22-27`, `contracts/project/src/lib.rs:27-33`, `contracts/deployment/src/lib.rs:28-34`, `contracts/permissions/src/lib.rs:20-25`, `contracts/registry/src/lib.rs:31-48`
**Difficulty:** Intermediate

---

# 2. Contracts: Architecture & Design

---

## Issue #132 — No cross-contract validation exists between contracts

**Labels:** feature, contracts

**Summary:**
- `Project::register` accepts any `org_id` without verifying the org exists in the Organization contract
- `Deployment::record` accepts any `project_id` without verifying the project exists
- `Permissions::grant_role` accepts any `org_id` without verifying the org exists
- README architecture diagram shows cross-contract verification arrows, but no cross-contract calls exist

**Acceptance Criteria:**
- [ ] Decide validation strategy (cross-contract vs off-chain)
- [ ] If cross-contract: implement `Env::invoke_contract` for parent existence checks
- [ ] If off-chain: document the assumption explicitly

**Difficulty:** Advanced

---

## Issue #133 — No contract upgrade mechanism implemented

**Labels:** feature, contracts

**Summary:**
Soroban supports `env.deployer().update_current_contract_wasm()` for contract upgrades. None of the 5 contracts implement this. Once deployed, contracts are permanently immutable. README roadmap lists "Contract upgrade support" but no upgrade function, storage key, migration pattern, or governance mechanism exists.

**Acceptance Criteria:**
- [ ] Add upgrade function to each contract (guarded by admin/owner)
- [ ] Define storage migration patterns for future versions
- [ ] Add upgrade governance (timelock or multisig)

**Difficulty:** Advanced

---

## Issue #134 — No pause mechanism for any contract

**Labels:** feature, contracts

**Summary:**
No contract implements a pause/unpause mechanism. If a critical vulnerability is discovered, there is no way to halt state-changing functions without upgrading (which itself isn't implemented — Issue #133). All contracts are always "live."

**Acceptance Criteria:**
- [ ] Add `paused` boolean to each contract's storage
- [ ] Add `pause()` / `unpause()` functions guarded by admin
- [ ] Check `!paused` in all state-changing functions

**Difficulty:** Intermediate

---

## Issue #135 — Deployment version counter can overflow `u32`

**Labels:** bug, contracts

**Summary:**
`deployment/lib.rs:57` — `let version = count + 1;` — when `count` reaches `u32::MAX`, the `+1` panics (with `overflow-checks = true`). No `checked_add`, no error handling. An admin who records 4 billion deployments causes a panic.

**Acceptance Criteria:**
- [ ] Use `count.checked_add(1).ok_or(ContractError::StorageFailure)?`
- [ ] Add test for `u32::MAX` edge case

**Difficulty:** Intermediate

---

## Issue #136 — Registry duplicates data without source-of-truth verification

**Labels:** feature, contracts

**Summary:**
Registry stores copies of OrgInfo, ProjectInfo, DeploymentInfo but never calls the source contracts to verify data. The three contract addresses in `RegConfig` (`org_contract`, `project_contract`, `deployment_contract`) are stored but never read. The registry is a manually-populated cache with no consistency guarantees.

**Acceptance Criteria:**
- [ ] Option A: Implement cross-contract verification on index
- [ ] Option B: Remove unused contract address fields from Config
- [ ] Option C: Add events so off-chain indexers can verify independently

**Difficulty:** Intermediate

---

## Issue #137 — No `get_latest_deployment` function exists

**Labels:** feature, contracts

**Summary:**
`Deployment::get` requires both `project_id` and `version`. To get the latest deployment, callers must perform two calls: `count(project_id)` then `get(project_id, count)`. Race condition possible between them.

**Acceptance Criteria:**
- [ ] Add `get_latest(project_id)` to Deployment contract
- [ ] Returns `NotFound` if no deployments exist
- [ ] Or add `lookup_latest_deployment` to Registry

**Difficulty:** Beginner

---

## Issue #138 — Storage read before authorization check in multiple functions

**Labels:** refactor, contracts

**Summary:**
In `Organization`, `Project`, and `Deployment` contracts, storage reads (e.g., `read_org`, `read_project`) occur before `authorize()` checks. An unauthorized caller causes a wasteful storage read:
- `organization/lib.rs:59-61` — read before authorize
- `organization/lib.rs:76-78` — read before authorize
- `organization/lib.rs:86-88` — read before authorize
- `project/lib.rs:78-79` — read before authorize

**Acceptance Criteria:**
- [ ] Move authorization checks before storage reads where possible
- [ ] Verify no authorization bypass is created by the reorder

**Difficulty:** Intermediate

---

# 3. Security & Auth

---

# 3. Contracts: Security & Auth

---

## Issue #139 — `auth::authorize` is a zero-value wrapper around `require_auth`

**Labels:** refactor, contracts

**Summary:**
`shared/src/auth.rs:5-8` — `authorize(owner: &Address) -> Result<()>` calls `owner.require_auth(); Ok(())`. This is literally 1 line of meaningful code inside a function. It increases WASM size and adds indirection. The Registry and Permissions contracts call `require_auth()` directly anyway, making the pattern inconsistent.

**Acceptance Criteria:**
- [ ] Remove the wrapper, call `require_auth()` directly everywhere
- [ ] Or rename to `require_auth` and ensure all contracts use it

**Difficulty:** Beginner

---

## Issue #140 — `check_active` returns `InvalidOrganization` even when checking project status

**Labels:** bug, contracts

**Summary:**
`shared/src/auth.rs:10-14` — `check_active` always returns `ContractError::InvalidOrganization` (error code 5) regardless of entity type. When called from `project::archive` to check a project's archived status, it returns "Invalid Organization" — semantically wrong.

**Acceptance Criteria:**
- [ ] Make error type parameterizable, or create `check_org_active` and `check_project_active`
- [ ] Update all call sites

**Difficulty:** Intermediate

---

## Issue #141 — `grant_role` requires dual auth but `revoke_role` only requires admin auth

**Labels:** design, contracts

**Summary:**
`permissions/lib.rs:39-40` — `grant_role` requires both `admin.require_auth()` and `user.require_auth()`. `revoke_role` at line 63 only requires `admin.require_auth()`. The user cannot prevent their own role from being revoked — intentional but asymmetric and potentially confusing.

**Acceptance Criteria:**
- [ ] Document the asymmetry explicitly in the contract README
- [ ] Consider whether `revoke_role` should also require user auth

**Difficulty:** Beginner

---

## Issue #142 — `Role` enum has no validation for arbitrary u32 values

**Labels:** bug, contracts

**Summary:**
`shared/types.rs:34-41` — `Role` is `Owner=0, Admin=1, Developer=2, Viewer=3`. If a caller passes a raw u32 of 4+ as a role (via Soroban serialization), behavior is undefined. `ContractError::InvalidRole` exists (error code 4) but is never used.

**Acceptance Criteria:**
- [ ] Validate role value in `grant_role`: return `InvalidRole` if `role > 3`
- [ ] Remove unused error variant or use it for this validation

**Difficulty:** Intermediate

---

## Issue #143 — `revoke_role` stores `role` key with `Address` as part of composite key — clone overhead

**Labels:** refactor, contracts

**Summary:**
`permissions/lib.rs:76` — `user.clone()` — the `Address` is cloned unnecessarily. It's the last use of `user` in the function (consumed by `remove`), so it could be moved.

**Acceptance Criteria:**
- [ ] Remove the `clone()` — let `user` be consumed by `remove`

**Difficulty:** Beginner

---

# 4. Testing Quality

---

# 4. Contracts: Testing Quality

---

## Issue #144 — All tests except one use `mock_all_auths()` — auth behavior never validated

**Labels:** testing, contracts

**Summary:**
Only 1 of 30 tests (`test_transfer_unauthorized` in Organization) clears auths to verify authorization. All other tests use blanket `mock_all_auths()` which bypasses real auth checking. Auth enforcement on 4/5 contracts is completely untested.

**Acceptance Criteria:**
- [ ] Every state-changing function has at least one test with unauthorized caller
- [ ] Expected error: `ContractError::Unauthorized` or panic

**Difficulty:** Intermediate

---

## Issue #145 — No contract has a test for `init` re-initialization panic

**Labels:** testing, contracts

**Summary:**
All 5 contracts panic on re-init (`panic!("already initialized")`). No test verifies this behavior. If a future change accidentally makes init idempotent, no test catches it.

**Acceptance Criteria:**
- [ ] Each contract has a test: `init` → `init` → expect panic
- [ ] Use `#[should_panic(expected = "already initialized")]` pattern

**Difficulty:** Beginner

---

## Issue #146 — Registry tests have weak assertions — single field checked

**Labels:** testing, contracts

**Summary:**
`registry/test.rs:60,70,80` — each `lookup_*` test only asserts one field of the returned struct. For example, `lookup_org` only checks `owner`, not `metadata_hash`, `created_at`, or `status`.

**Acceptance Criteria:**
- [ ] Assert all fields of returned struct in every lookup test
- [ ] Match against the exact input value for each field

**Difficulty:** Beginner

---

## Issue #147 — No test for `Registry::index_deployment` version tracking

**Labels:** testing, contracts

**Summary:**
Registry's `DeploymentCount` version-tracking logic is untested: (1) count starts at 0, (2) indexing version 1 sets count to 1, (3) indexing version 5 then version 3 keeps count at 5, (4) lookup for non-existent deployment returns error.

**Acceptance Criteria:**
- [ ] Test all 4 scenarios above

**Difficulty:** Intermediate

---

## Issue #148 — `Deployment::test_get_not_found` uses uninitialized contract

**Labels:** testing, contracts

**Summary:**
`deployment/test.rs:44-49` — creates a fresh contract registration without calling `init`, then calls `get`. While this works (get doesn't check config), it tests an uninitialized contract rather than a properly initialized one with a missing deployment.

**Acceptance Criteria:**
- [ ] Initialize the contract, then test `get` for a non-existent deployment

**Difficulty:** Beginner

---

## Issue #149 — Missing test for archiving an already-archived project

**Labels:** testing, contracts

**Summary:**
`project::archive` calls `check_active` which should reject archiving an already-archived project. No test verifies this error path.

**Acceptance Criteria:**
- [ ] Test: register → archive → archive → expect `InvalidOrganization` (or `InvalidProject` after Issue #140 fix)

**Difficulty:** Beginner

---

## Issue #150 — `has_role` with non-existent user returns false but no explicit test

**Labels:** testing, contracts

**Summary:**
`permissions::has_role` returns `false` for users without a role. This is correct behavior but the test suite has no explicit test for it.

**Acceptance Criteria:**
- [ ] Test that `has_role` returns `false` for user with no role

**Difficulty:** Beginner

---

## Issue #151 — Soroban snapshot tests are fragile — any test setup change invalidates all snapshots

**Labels:** testing, contracts

**Summary:**
30 snapshot JSON files contain deterministic but fragile addresses and nonces. Any change to test setup order (number of addresses generated, order of tests, Soroban SDK version) breaks all snapshots.

**Acceptance Criteria:**
- [ ] Document snapshot maintenance in CONTRIBUTING.md
- [ ] Consider whether snapshots provide value beyond regular assertions

**Difficulty:** Beginner

---

# 5. Events & Storage

---

# 5. Contracts: Events & Storage

---

## Issue #152 — Registry contract emits no events for any state change

**Labels:** feature, contracts

**Summary:**
`registry/lib.rs:50-111` — `index_org`, `index_project`, `index_deployment` all mutate storage but emit zero events. Every other contract emits events. Off-chain indexers cannot detect registry updates without polling.

**Acceptance Criteria:**
- [ ] Add event emission to all three `index_*` functions
- [ ] Follow `symbol_short!` topic key pattern from `events.rs`

**Difficulty:** Beginner

---

## Issue #153 — `update_metadata` in Organization contract emits no event

**Labels:** feature, contracts

**Summary:**
`organization/lib.rs:71-83` — metadata hash is updated silently. All other Organization mutations emit events. Off-chain systems have no way to know metadata changed.

**Acceptance Criteria:**
- [ ] Add event emission to `update_metadata`

**Difficulty:** Beginner

---

## Issue #154 — Registry `index_org`/`index_project` allow silent overwrite without check

**Labels:** bug, contracts

**Summary:**
Unlike `Organization::create` (which returns `AlreadyExists`), Registry's `index_org` and `index_project` silently overwrite existing entries. This can mask data inconsistency between source contracts and the registry.

**Acceptance Criteria:**
- [ ] Decide whether overwrite is intentional (re-indexing support) or a bug
- [ ] If intentional: document it and add event with overwrite indicator
- [ ] If bug: emit `AlreadyExists` on duplicate index

**Difficulty:** Intermediate

---

## Issue #155 — Config read from storage on every function call — wasteful for read-only functions

**Labels:** performance, contracts

**Summary:**
Every function reads config from instance storage on each invocation (e.g., `registry::lookup_org` at line 114 calls `read_config` but doesn't use the admin address for a read-only lookup). For read-only `lookup_*` functions, the config fetch is unnecessary.

**Acceptance Criteria:**
- [ ] Separate read-only functions from state-changing functions — don't fetch config in read-only paths

**Difficulty:** Intermediate

---

## Issue #156 — Event symbol names use 8-char truncation that loses readability

**Labels:** refactor, contracts

**Summary:**
`shared/events.rs` — `symbol_short!` truncates to 8 characters: `org_creat`, `org_xfer`, `org_archv`, `proj_crea`, `dep roc` (separate), `proj_arch`, `deploy_re`, `role_grnt`, `role_revk`. These are hard to read and debug.

**Acceptance Criteria:**
- [ ] Use `Symbol::new` with full names instead of `symbol_short!`
- [ ] Or document the truncation table clearly in README

**Difficulty:** Beginner

---

## Issue #157 — `role_granted` and `role_revoked` events have inconsistent structure

**Labels:** bug, contracts

**Summary:**
- `role_granted`: topic `(ROLE_GRANT, org_id, user)`, data = `role`
- `role_revoked`: topic `(ROLE_REVOK, org_id)`, data = `user`

The user is in the topic for grants but in the data for revocations. Off-chain indexers cannot filter revocations by user.

**Acceptance Criteria:**
- [ ] Make both events consistent: put user in topic or data in both

**Difficulty:** Intermediate

---

# 6. Documentation & Config

---

# 6. Contracts: Documentation & Config

---

## Issue #158 — README license badge says MIT but actual license is Apache-2.0

**Labels:** documentation, contracts

**Summary:**
`README.md:4` — badge links to MIT icon. `LICENSE` file is Apache 2.0. `Cargo.toml:15` correctly says `license = "Apache-2.0"`. Badge is misleading.

**Acceptance Criteria:**
- [ ] Update badge to `[![License: Apache 2.0]`

**Difficulty:** Beginner

---

## Issue #159 — README claims 30+ tests but there are exactly 30

**Labels:** documentation, contracts

**Summary:**
8 + 5 + 5 + 7 + 5 = 30 tests. README claims "30+ Tests" at lines 29, 368, 492. The "+" is misleading.

**Acceptance Criteria:**
- [ ] Change to "30 tests" or add more tests to exceed 30

**Difficulty:** Beginner

---

## Issue #160 — README "No Unchecked Arithmetic" claim is misleading

**Labels:** documentation, contracts

**Summary:**
README:482 claims "Soroban SDK provides safe arithmetic; deployment versions auto-increment safely." The deployment `count + 1` can overflow (Issue #135). Soroban SDK does not provide safe arithmetic by default — `overflow-checks = true` in release profile causes a panic, not safe handling.

**Acceptance Criteria:**
- [ ] Fix the documentation to accurately describe overflow behavior
- [ ] Fix the code (Issue #135) and update docs

**Difficulty:** Beginner

---

## Issue #161 — `.env.example` comment says "base64-encoded" for Stellar secret key

**Labels:** bug, documentation

**Summary:**
`.env.example:8` — `# Deployer secret key (base64-encoded)` — Stellar secret keys are base32-encoded (starting with `S`), not base64.

**Acceptance Criteria:**
- [ ] Fix comment to say "base32-encoded Stellar secret key"

**Difficulty:** Beginner

---

## Issue #162 — README error sources table inaccurate

**Labels:** documentation, contracts

**Summary:**
README error codes table claims:
- Project uses `InvalidOrganization` (it does via `check_active`, but that's a bug — Issue #140)
- Registry claims `Unauthorized` but uses direct `require_auth()` which panics instead of returning a proper error

**Acceptance Criteria:**
- [ ] Audit error table against actual code
- [ ] Fix inaccuracies

**Difficulty:** Beginner

---

## Issue #163 — `#![allow(deprecated)]` in shared crate suppresses all deprecation warnings

**Labels:** refactor, contracts

**Summary:**
`shared/src/lib.rs:2` — crate-level `#![allow(deprecated)]`. Suppresses ALL deprecation warnings for every contract. If Soroban SDK 27.0.0 deprecates any API used, developers won't see the warning.

**Acceptance Criteria:**
- [ ] Identify the specific deprecated item that triggered this
- [ ] Replace with scoped `#[allow(deprecated)]` on the specific expression
- [ ] If no specific item, remove the allow entirely

**Difficulty:** Intermediate

---

# 7. CI/CD & Tooling

---

# 7. Contracts: CI/CD & Tooling

---

## Issue #164 — Release and CD workflows create duplicate GitHub Releases

**Labels:** bug, contracts, devops

**Summary:**
`release.yml` creates a GitHub Release (using `softprops/action-gh-release@v2`) and pushes a tag. `cd.yml` triggers on tag push (`v*`) and creates a **second** GitHub Release with WASM artifacts. These conflict — both upload to the same tag, causing race conditions.

**Acceptance Criteria:**
- [ ] Release workflow should only tag and bump version
- [ ] CD workflow triggered by tag should create the actual release with artifacts

**Difficulty:** Intermediate

---

## Issue #165 — `release.yml` sed command is fragile for version bump

**Labels:** bug, contracts, devops

**Summary:**
`release.yml:21` — `sed -i "s/^version = \".*\"/.../" Cargo.toml` — matches any line starting with `version =`. If a dependency line starts with `version =`, it gets corrupted. Also doesn't update CHANGELOG.md.

**Acceptance Criteria:**
- [ ] Use more specific sed pattern (match `[package]` section only)
- [ ] Add CHANGELOG.md update step

**Difficulty:** Intermediate

---

## Issue #166 — `audit.yml` missing WASM target installation

**Labels:** bug, contracts, devops

**Summary:**
`audit.yml` doesn't install `wasm32v1-none` target in the `dtolnay/rust-toolchain` step (unlike `ci.yml`). `deny.toml` targets `wasm32v1-none` — `cargo deny` may give incomplete results.

**Acceptance Criteria:**
- [ ] Add `targets: wasm32v1-none` to toolchain setup in audit.yml

**Difficulty:** Beginner

---

## Issue #167 — No WASM optimization step in CI or build

**Labels:** feature, contracts, devops

**Summary:**
Release profile uses `opt-level = "z"` and `lto = true` but no `wasm-opt` (Binaryen) post-processing. WASM files are uploaded as-is without size optimization.

**Acceptance Criteria:**
- [ ] Add `wasm-opt -Oz` step after WASM build in CI
- [ ] Report WASM file sizes in CI output

**Difficulty:** Intermediate

---

## Issue #168 — CI cargo-deny installed fresh every run (no caching)

**Labels:** devops, contracts

**Summary:**
`ci.yml:60-61` and `audit.yml:16-17` — `cargo install cargo-deny --locked` runs every time, adding 2-3 minutes per workflow.

**Acceptance Criteria:**
- [ ] Cache `cargo-deny` binary or use `cargo-binstall`

**Difficulty:** Beginner

---

## Issue #169 — No TypeScript/JSON contract bindings generation

**Labels:** feature, contracts, devops

**Summary:**
No workflow step or script generates `soroban contract bindings` output for TypeScript or JSON. Frontend/backend must manually construct contract calls.

**Acceptance Criteria:**
- [ ] Add bindings generation step to CI
- [ ] Output uploaded as CI artifact

**Difficulty:** Intermediate

---

## Issue #170 — Empty `scripts/`, `docs/`, `tests/` directories

**Labels:** cleanup, contracts

**Summary:**
Three directories exist with zero files. They communicate intent but contain nothing.

**Acceptance Criteria:**
- [ ] Populate with content or remove

**Difficulty:** Beginner

---

# 19. Cross-Cutting Testing

---

# 8. Cross-Cutting Testing

---

## Issue #171 — Backend has zero integration or API tests

**Labels:** testing, backend

**Summary:**
`tests/integration/mod.rs`, `tests/api/mod.rs`, `tests/unit/mod.rs` are all doc-only. No integration test functions exist. The only tests are 22 inline unit tests in `auth/challenge.rs`, `auth/wallet.rs`, `permissions/role.rs`, `permissions/policy.rs`, and `config/tests.rs`.

**Acceptance Criteria:**
- [ ] Integration test for auth flow (challenge → login → refresh → logout)
- [ ] Integration test for org CRUD
- [ ] API contract tests for all implemented endpoints
- [ ] Repository tests with test database

**Difficulty:** Advanced

---

## Issue #172 — `tests/common/test_app.rs` won't compile due to wrong function signature

**Labels:** bug, backend, testing

**Summary:**
`test_app.rs:19-24` — `TestAppBuilder::build()` calls `router::build_router(config)` with one argument, but `build_router` requires TWO arguments (`config`, `state`). This will not compile. The test suite is completely non-functional.

**Acceptance Criteria:**
- [ ] Fix function signature to match `router::build_router`
- [ ] Verify `cargo test` compiles and runs

**Difficulty:** Intermediate

---

## Issue #173 — No E2E tests for authenticated flows in frontend

**Labels:** testing, frontend

**Summary:**
Single E2E test only covers landing page. No test verifies: middleware redirect, login flow, dashboard rendering, navigation between pages.

**Acceptance Criteria:**
- [ ] E2E test for authenticated dashboard (inject cookie)
- [ ] E2E test for middleware redirect when unauthenticated

**Difficulty:** Advanced

---

## Issue #174 — No property-based or fuzz tests for any component

**Labels:** testing

**Summary:**
`proptest` is in backend's dev-dependencies but never used. Contracts deal with numeric IDs, timestamps, and hash values that would benefit from property-based invariants. No fuzz testing exists.

**Acceptance Criteria:**
- [ ] Property-based test for deployment version monotonicity
- [ ] Property-based test for permission hierarchy consistency
- [ ] Property-based test for pagination param bounds

**Difficulty:** Advanced

---

## Issue #175 — No a11y tests or automated accessibility audit

**Labels:** testing, accessibility

**Summary:**
`@storybook/addon-a11y` is installed but no a11y stories exist. No `axe-core` integration in Playwright or Vitest.

**Acceptance Criteria:**
- [ ] Add `axe-playwright` as dev-dependency
- [ ] Add a11y test for landing page and dashboard page

**Difficulty:** Intermediate

---

## Issue #176 — No load or stress tests for backend or contracts

**Labels:** testing

**Summary:**
No benchmarks, load tests, or stress tests exist. No data on how the API performs under concurrent load, how the job queue handles backpressure, or how contracts handle high-throughput scenarios.

**Acceptance Criteria:**
- [ ] `k6` or `locust` test for top 5 API endpoints
- [ ] Contract stress test (max storage size, max event count)

**Difficulty:** Advanced

---

# 20. Developer Experience

---

# 9. Developer Experience

---

## Issue #177 — Frontend CI workflows may be empty or missing

**Labels:** devops, frontend

**Summary:**
`.github/workflows/` — CI/CD workflow files may exist but appear empty based on audit. No automated quality checks run on push/PR.

**Acceptance Criteria:**
- [ ] Verify `ci.yml` contains `typecheck`, `lint`, `test`, `build` jobs
- [ ] Verify `cd.yml` contains Docker build and push
- [ ] README badges reflect actual CI status

**Difficulty:** Beginner

---

## Issue #178 — No Rust/cargo caching in any CI workflow

**Labels:** devops

**Summary:**
Backend CI, contracts CI, and all CD/release workflows build from scratch with no cargo caching. Full build takes 15-30 minutes.

**Acceptance Criteria:**
- [ ] Add `Swatinem/rust-cache` action to all Rust CI workflows
- [ ] Verify cache hit reduces build time to under 5 minutes

**Difficulty:** Beginner

---

## Issue #179 — No Makefile, Justfile, or task runner for common commands

**Labels:** devops

**Summary:**
Backend and contracts have no task runner. Developers must remember exact commands: `cargo build --features default`, `cargo test --workspace`, `cargo clippy --all-targets --all-features`. The `scripts/` directory in contracts is empty.

**Acceptance Criteria:**
- [ ] Create Makefile with targets: `build`, `test`, `lint`, `check`, `clean`, `docker-build`
- [ ] Include WASM build shortcut for contracts

**Difficulty:** Beginner

---

## Issue #180 — `commitlint.config.mjs` uses `sentence-case` — incompatible with conventional commits

**Labels:** bug, devops

**Summary:**
`commitlint.config.mjs:5` — `'subject-case': [2, 'always', 'sentence-case']` — conventional commits typically use lowercase subjects (e.g., `feat: add user login`). This rule will cause CI failures for standard-format commits.

**Acceptance Criteria:**
- [ ] Change to `'lower-case'` or remove the rule

**Difficulty:** Beginner

---

## Issue #181 — Frontend ESLint missing React Hooks plugin rules

**Labels:** devops, frontend

**Summary:**
`eslint.config.mjs` — no `react-hooks/exhaustive-deps` rule. Missing dependency warnings in `useEffect`/`useMemo`/`useCallback` won't be caught. Combined with `--max-warnings 0`, this means code with missing deps can pass lint.

**Acceptance Criteria:**
- [ ] Add `eslint-plugin-react-hooks` rules
- [ ] Address any new warnings

**Difficulty:** Beginner

---

## Issue #182 — Frontend `tsconfig.json` doesn't catch unused variables

**Labels:** devops, frontend

**Summary:**
`tsconfig.json` — `strict: true` but no `noUnusedLocals` or `noUnusedParameters`. Dead code and unused params are not caught by `tsc --noEmit`.

**Acceptance Criteria:**
- [ ] Enable `noUnusedLocals` and `noUnusedParameters`
- [ ] Address any new errors

**Difficulty:** Intermediate

---

## Issue #183 — Storybook and e2e directories excluded from TypeScript checking

**Labels:** devops, frontend

**Summary:**
`tsconfig.json:41-42` — excludes `stories` and `e2e` directories. Type errors in story files and E2E tests go unnoticed.

**Acceptance Criteria:**
- [ ] Include stories and e2e in type-checking (they are small directories)

**Difficulty:** Beginner

---

## Issue #184 — `package.json` `check` script runs sequentially: typecheck → lint → test → build

**Labels:** devops, frontend

**Summary:**
`check` script takes 3x as long as necessary. These steps are independent and could run in parallel.

**Acceptance Criteria:**
- [ ] Use `concurrently` (or `npm-run-all`) to run steps in parallel

**Difficulty:** Beginner

---

## Issue #185 — No `.env.example` files for local development in backend/frontend

**Labels:** documentation

**Summary:**
Backend and frontend do not ship `.env.example` files. Contributors must guess or dig through config code to find required environment variables.

**Acceptance Criteria:**
- [ ] Create `.env.example` for backend with all config variables documented
- [ ] Create `.env.example` for frontend (one may exist, verify completeness)

**Difficulty:** Beginner

---

## Issue #186 — No ADR documents exist beyond template

**Labels:** documentation

**Summary:**
`docs/ADR/` contains a template and two ADRs (use-axum, use-sqlx) but many significant decisions are undocumented: why postgres over mysql, why event bus pattern, why push-based registry pattern, why separate trait/impl pattern.

**Acceptance Criteria:**
- [ ] Add ADRs for: event bus choice, registry architecture, service/repository pattern, deployment state machine design, wallet auth pattern

**Difficulty:** Intermediate
