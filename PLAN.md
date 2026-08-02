# m2s2-cli — Execution Plan

## Overview

`m2s2` is a Rust CLI tool for scaffolding and managing projects that use the M²S² design system,
and for preparing and deploying canonical content through the M2S2 publishing platform. It resolves
live npm package versions at scaffold time, embeds all templates as compiled binaries, and
self-updates via GitHub Releases.

For content delivery, the CLI is evolving into a hybrid client: it owns local Markdown parsing,
validation, rendering, hashing, and user interaction, while the Go platform owns destination
credentials, remote preflight, asynchronous orchestration, adapters, retries, approvals,
publication state, and analytics. See `docs/platform-content-delivery-architecture.md`. The
offline half of this — the canonical article schema and `m2s2 content init/validate/inspect` — is
done (Phase 6a); the platform-dependent half (`auth`, `workspace`, remote `content
generate/publish/status/retry/approve`) is still blocked on the unbuilt Go platform API
(Phase 6b).

**Repo**: `M2S2-Engineering-Group/m2s2-cli`
**Published**: crates.io + npm (`@m2s2/cli`)
**Install**: `npm i -g @m2s2/cli` or the shell/PowerShell installer from cargo-dist

---

## Current Commands

### `m2s2 new <name>`
Scaffolds a project. Frontend (Angular/React/Vue), backend (Go/Node/Python across
Gin/Echo/Fiber/Express/Fastify/FastAPI/Flask), or fullstack (frontend in `apps/web` +
backend in `apps/api`), optionally with AWS Cognito auth, Stripe billing, a CDK stack, and
a GitHub Actions workflow. Prompts interactively for anything not passed via flags. Resolves
all dependency versions live from npm (using the m2s2 lib as source of truth), writes all
files from embedded Handlebars templates, runs the install step (`npm install` /
`go mod tidy` / `python3 -m venv` + `pip install`), and writes a `.m2s2.json` project config.

**Templates**: `templates/{angular,react,vue,gin,echo,fiber,express,fastify,fastapi,flask,fullstack,auth,billing,cdk,github-actions,root}/`
**Flags**: `--project-type`, `--framework`, `--runtime`, `--api-framework`, `--auth`, `--billing`, `--skip-install`, `--offline`

### `m2s2 generate component|page|service <name>`
Scaffolds into the current project. Auto-detects framework from `.m2s2.json` (written by
`new`) or `package.json` (`@m2s2/ng-lib`, `@m2s2/react-lib`, `@m2s2/vue-lib`).

**component** — `src/app/components/<kebab>/` (Angular) or `src/components/<Pascal>/` (React/Vue)
**page** — `src/app/pages/<kebab>/` (Angular) or `src/pages/<Pascal>/` (React/Vue); prints a routing snippet
**service** — Angular only, `@Injectable` stub in `src/app/services/<kebab>/`; React/Vue are told to write a hook/composable instead

**Flags**: `--framework`, `--path` (component/page); `--path` (service)

### `m2s2 dev|build|test|lint`
Unified project commands that dispatch based on `.m2s2.json`'s `project_type`/`framework`/
`runtime`: run the frontend's npm script, the backend's native tool (`go`/npm/`python3 -m`),
or both concurrently (`dev`) / sequentially (`build`/`test`/`lint`) for fullstack. Extra args
pass through to the underlying tool.

### `m2s2 upgrade [--check]`
Fetches latest release from GitHub API, compares semver, and runs the cargo-dist installer
script to update in-place. `--check` reports available version without installing.

### `m2s2 completions [shell]`
Generates shell completions and wires them into the appropriate RC file. Auto-detects from
`$SHELL`. Supports zsh, bash, fish, elvish, PowerShell.

### `m2s2 publish <file.md>`
Publishes a Markdown article with YAML frontmatter to DEV, Hashnode, and/or a generic platform
target. The current implementation executes adapters directly from the CLI using
`.m2s2-publish.toml`. This remains supported during the platform migration, but will become a
compatibility alias for `m2s2 content publish` once remote execution is proven.

**Current layout**: `src/publish/` with command glue in `src/commands/publish.rs`.
**Migration boundary**: local preparation remains in the CLI; remote credentials, preflight,
delivery, retries, approvals, and state move to the Go platform.

### `m2s2 content init|validate|inspect <file>`
Authors and validates the canonical content-delivery article schema — a schema deliberately kept
separate from `publish`'s (requires `canonical_url`, drops `date`/`targets`). Offline only: no
network access, no credentials required. `init` writes `.m2s2/config.toml` plus `articles_dir`/
`assets_dir`; `validate`/`inspect` run every offline rule (required fields, HTTPS canonical URL,
duplicate slugs, local-path escape protection, unresolved placeholders, schema version) and print
a human or `--format json` report.

**Current layout**: `src/content/` with command glue in `src/commands/content.rs`; shared report
type in `src/report.rs`.
**Migration boundary**: remote-dependent subcommands (`content publish`, `status`, `retry`,
`approve`) are Phase 6b, blocked on the platform API.

---

## Release Pipeline

1. Push `feat`/`fix`/`perf` to `main` → CI passes → `release-plz.yml` bumps `Cargo.toml`, commits, pushes tag
2. Tag push → `release.yml` (cargo-dist) → builds platform binaries + npm package → GitHub Release
3. GitHub Release published → `publish-npm.yml` → `npm publish`
4. Tag push → `publish-crates.yml` → `cargo publish`

`chore`, `ci`, `docs`, `refactor`, `test`, `build` commits do **not** trigger a version bump.

---

## Phase 1 — Current State ✅

- [x] `new` command — frontend / backend / fullstack, across all supported frameworks/runtimes
- [x] Optional auth (Cognito), billing (Stripe), CDK stack, GitHub Actions workflow scaffolding
- [x] `generate component` / `generate page` / `generate service` with framework auto-detection
- [x] `dev` / `build` / `test` / `lint` — unified commands dispatching to the right underlying tool
- [x] `upgrade` with GitHub release check + installer
- [x] `completions` with RC file injection
- [x] Live npm version resolution at scaffold time
- [x] Embedded Handlebars templates (compiled into binary)
- [x] Fully automated release pipeline (CI-gated, no PRs, no manual steps)
- [x] `.m2s2.json` project config (see Phase 3, below — already implemented, not just planned)
- [x] Direct `publish` command for DEV, Hashnode, and a configurable platform target
- [x] YAML frontmatter article parsing and target-specific cover-image handling
- [x] Mocked HTTP contract tests for the current publishing adapters
- [x] Canonical content-delivery article schema + `content init/validate/inspect` (Phase 6a,
  offline only — see below)
- [x] One starter test per stack (react/vue/angular/express/fastify/fastapi/flask — hits
  `/health` or renders `App`) so `m2s2 test` passes out of the box instead of failing on an
  empty test suite. Go already passed trivially (`go test` with no test files is not an error).

---

## Phase 2 — Generate Subcommands

`generate component` / `page` / `service` are done (see Phase 1). Still open:

### `generate story <name>`
Generates a Storybook story file alongside an existing component.

- Detects the component file, infers framework, writes `<Name>.stories.tsx` / `.stories.ts`
- Includes default story and one variant story stub

### `generate test <name>`
Generates a test/spec file alongside an existing (non-root) component, mirroring the starter
tests added in Phase 1.

- **Angular**: `<kebab>.component.spec.ts` using `TestBed`
- **React**: `<Pascal>.test.tsx` using Vitest + Testing Library
- **Vue**: `<Pascal>.test.ts` using Vitest + Testing Library

---

## Phase 3 — Project-Level Config ✅

Done — `.m2s2.json` is written by `new` and read by `generate`/`dev`/`build`/`test`/`lint`
(`src/config.rs`). Falls back to `package.json` dependency detection, then (for `generate`)
an explicit `--framework` flag.

```json
{
  "framework": "angular",
  "apiFramework": "gin",
  "projectType": "fullstack",
  "runtime": "go",
  "auth": false,
  "billing": false
}
```

---

## Phase 4 — Discovery & Diagnostics

### `m2s2 list`
Lists all components available in the version of the m2s2 lib installed in the current project.
Fetches metadata from the npm registry for the installed version and prints a component inventory.

```
$ m2s2 list
@m2s2/react-lib@2.3.1 — 20 components
  Navbar          Footer          Navbar
  StatusBadge     PageHeader      SectionHeader
  ...
```

### `m2s2 doctor`
Validates that the project is correctly set up to use the design system.

Checks:
- Correct m2s2 library is installed and version is not stale
- Peer dependencies (React, Vue, Angular) are at compatible versions
- SCSS/token imports are resolvable
- Reports `✓ ok` / `✗ issue` per check with actionable fix hints

---

## Phase 5 — Content Domain Stabilization

Refactor the existing direct-publish implementation without changing its user-visible behavior.

- Extract article parsing, validation, operation types, prepared requests, preflight reports, and
  publication outcomes from command glue.
- Add full-target preparation before the first external write.
- Add `--preflight-only`, `--offline`, and machine-readable report output.
- Validate DEV requests against pinned Forem OpenAPI and Hashnode operations against a pinned
  GraphQL schema.
- Add the required Forem v1 `Accept` header.
- Treat malformed or incomplete successful responses as errors.
- Introduce publication state containing remote IDs, URLs, source hashes, and target status.
- Retain `.m2s2-publish.toml` and current direct execution during this phase.

Related design: `docs/api-verification-preflight.md`.

---

## Phase 6a — Local Content Authoring ✅

The offline portion of the canonical content schema — no platform dependency, so it doesn't wait
on Phase 6b.

- [x] Canonical article schema (`src/content/article.rs`) — a schema deliberately kept separate
  from `publish::article::Article` (requires `canonical_url`, drops `date`/`targets`), sharing
  only genuinely schema-agnostic helpers (`src/markdown.rs`'s `split_frontmatter`/`slugify`,
  `publish::cover_image::resolve`) rather than a unified struct.
- [x] Offline validation rules: required fields, `canonical_url` HTTPS + matches the configured
  `canonical_base_url`, slug well-formedness + duplicate detection across `articles_dir`,
  cover-image/body-link path-escape protection (rejects local references that resolve outside
  `articles_dir`/`assets_dir`), unresolved-`{{ }}`-placeholder detection, `schema_version`
  compatibility.
- [x] Minimal `.m2s2/config.toml` (`src/content/config.rs`) — only `schema_version` and
  `[content]` (`articles_dir`, `assets_dir`, `canonical_base_url`); no `[delivery]` section until
  Phase 6b's platform schema actually exists.
- [x] `m2s2 content init` / `content validate` / `content inspect` — human and `--format json`
  output via a shared `OutputReport`/`CheckStatus` type (`src/report.rs`) designed to be reused
  by Phase 5's still-pending `publish --preflight-only --format json` rather than inventing a
  second incompatible shape.
- [ ] `content preview` (Markdown rendering) — deliberately deferred; independent of validation
  and out of this pass's scope.
- [ ] Source/asset SHA-256 digest + normalization parity with the Go platform — blocked on
  cross-repo golden fixtures shared with Go and Angular; inventing the algorithm alone risks a
  rewrite once the Go side is defined.

## Phase 6b — Platform API Client

Introduce the platform boundary while keeping direct publishing available. Blocked entirely on
the Go platform API, which does not exist yet (`docs/platform-content-delivery-architecture.md`
describes unbuilt server-side work in a separate repo).

### New command groups

```text
m2s2 auth login|status|logout
m2s2 workspace list|use

m2s2 content generate <file> --for <targets>
m2s2 content publish <file>
m2s2 content status [run-id] [--watch]
m2s2 content retry <run-id> --target <target>
m2s2 content approve <run-id> --target <target>
```

`content validate`/`content inspect` already exist per Phase 6a; this phase only adds the
remote-dependent subcommands above.

### Client responsibilities

- Add a versioned Rust client for the Go platform's OpenAPI contract.
- Authenticate only to the M2S2 platform; never download destination credentials.
- Add platform profiles and browser/device authentication suitable for a terminal client.
- Store refresh credentials in the OS credential manager; support a workspace-scoped
  `M2S2_TOKEN` for CI.
- Extend `.m2s2/config.toml` with a `[delivery]` section for safe, non-secret content policy.
- Upload canonical article releases and assets using source hashes and idempotency keys.
- Request remote preflight/deployment and display platform-owned run state.
- Mock the platform API in ordinary CLI tests; require no real platform account.

The platform API and domain are specified in
`docs/platform-content-delivery-architecture.md`.

---

## Phase 7 — Dual Execution Migration

Support explicit execution modes while the hosted workflow is dogfooded:

```bash
m2s2 content publish article.md --execution direct
m2s2 content publish article.md --execution remote
```

- `direct` uses the current Rust destination adapters and local credentials.
- `remote` uploads a release and delegates all remote activity to the platform.
- Print the selected execution mode before any side effect.
- Keep `m2s2 publish` as a compatibility alias.
- A repository with a connected workspace may opt into `execution = "remote"`.
- Do not silently change existing repositories from direct to remote.
- Compare direct and remote behavior against shared article/preflight fixtures.

Exit criteria: M2S2 admin-editor and CLI releases successfully use the platform for canonical
publication, DEV, Hashnode, LinkedIn/X draft approval, status, and per-target retry.

---

## Phase 8 — Platform-First Operation

After the remote path is stable:

- Make remote execution the default for newly initialized connected content workspaces.
- Remove destination secrets from new CLI configuration flows.
- Retain direct mode only for backward compatibility, adapter development, or self-hosting.
- Decide whether direct mode remains supported, moves to a separate package, or is deprecated.
- Keep local validation and preview available without authentication or network access.
- Maintain the CLI as one client of the platform API, allowing a future separately branded SaaS
  CLI to reuse the same Rust client and local content libraries.

---

## Key Architecture Notes

- All templates are embedded at compile time via `rust-embed` — no runtime file reads
- `npm.rs` resolves versions by fetching `/latest` metadata from the npm registry; the m2s2
  library's own `peerDependencies`/`dependencies` are the authoritative source of compatible versions
- Name conversion: `to_pascal_case` + `to_kebab_case` handle all input forms (kebab, snake, PascalCase)
- Framework/runtime detection checks `.m2s2.json` first, then falls back to `package.json`
  `dependencies` for `@m2s2/ng-lib`, `react-lib`, `vue-lib` (`src/config.rs`)
- `scaffold_into` (`src/scaffold/mod.rs`) auto-discovers every embedded file under
  `templates/<prefix>/` — dropping a new file into a template directory is enough to have it
  included in `new`; nothing else needs registering (except files under a nested `generate/`
  path, which are reserved for `generate component/page/service` and excluded from `new`)
- The CLI/platform boundary is API-first: the CLI stops after authenticated release/workflow
  requests; the Go platform owns destination side effects and durable state.
- Local validation must remain usable offline. Remote preflight is platform-authoritative because
  only the platform has destination credentials and current connection policy.
- Destination credentials and OAuth refresh tokens never cross into the browser or CLI once remote
  mode is enabled.
- The Go platform publishes versioned OpenAPI; the Rust client is generated or contract-tested
  against the pinned specification.
- Existing direct adapters remain a migration mechanism, not a second durable source of publishing
  state once platform-first operation is adopted.
