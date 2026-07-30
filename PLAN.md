# m2s2-cli — Execution Plan

## Overview

`m2s2` is a Rust CLI tool for scaffolding and managing projects that use the M²S² design system.
It resolves live npm package versions at scaffold time, embeds all templates as compiled binaries,
and self-updates via GitHub Releases.

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
