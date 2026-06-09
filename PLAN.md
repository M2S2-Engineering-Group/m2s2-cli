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
Scaffolds a full project for Angular, React, or Vue. Prompts for framework if not passed via
`--framework`. Resolves all dependency versions live from npm (using the m2s2 lib as source of
truth), writes all files from embedded Handlebars templates, and runs `npm install`.

**Templates**: `templates/angular/`, `templates/react/`, `templates/vue/`
**Flags**: `--framework`, `--skip-install`

### `m2s2 generate component <name>`
Scaffolds a component into the current project. Auto-detects framework from `package.json`
(`@m2s2/ng-lib`, `@m2s2/react-lib`, `@m2s2/vue-lib`). Creates framework-appropriate files
in the correct default directory.

**Angular**: `.component.ts`, `.component.html`, `.component.scss` → `src/app/components/<kebab>/`
**React**: `.tsx`, `.scss`, `index.ts` → `src/components/<Pascal>/`
**Vue**: `.vue`, `.scss`, `index.ts` → `src/components/<Pascal>/`

**Flags**: `--framework`, `--path`

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

- [x] `new` command with Angular / React / Vue support
- [x] `generate component` with framework auto-detection
- [x] `upgrade` with GitHub release check + installer
- [x] `completions` with RC file injection
- [x] Live npm version resolution at scaffold time
- [x] Embedded Handlebars templates (compiled into binary)
- [x] Fully automated release pipeline (CI-gated, no PRs, no manual steps)

---

## Phase 2 — Generate Subcommands

Expand `generate` with additional scaffolding targets that mirror common project tasks.

### `generate page <name>`
Scaffold a full page/route entry — not just a component file.

- **Angular**: `.component.ts` + `.html` + `.scss` in `src/app/pages/<kebab>/`, with a route constant exported
- **React**: `.tsx` + `.scss` in `src/pages/<Pascal>/`, with a `<Route>` snippet printed to stdout
- **Vue**: `.vue` + `.scss` in `src/pages/<Pascal>/`, with a router entry snippet printed to stdout

### `generate service <name>` (Angular) / `generate hook <name>` (React, Vue)
- **Angular**: generates an `@Injectable` service stub in `src/app/services/<kebab>/`
- **React**: generates a `use<Name>.ts` hook stub in `src/hooks/`
- **Vue**: generates a `use<Name>.ts` composable stub in `src/composables/`

CLI auto-dispatches based on detected framework.

### `generate story <name>`
Generates a Storybook story file alongside an existing component.

- Detects the component file, infers framework, writes `<Name>.stories.tsx` / `.stories.ts`
- Includes default story and one variant story stub

### `generate test <name>`
Generates a test/spec file alongside an existing component.

- **Angular**: `<kebab>.component.spec.ts` using `TestBed`
- **React**: `<Pascal>.test.tsx` using Vitest + Testing Library
- **Vue**: `<Pascal>.test.ts` using Vitest + Testing Library

---

## Phase 3 — Project-Level Config

Add an optional `.m2s2.json` config file at the project root to store per-project defaults,
eliminating the need to pass `--framework` and `--path` on every `generate` invocation.

```json
{
  "framework": "angular",
  "componentPath": "src/app/features",
  "pagePath": "src/app/pages"
}
```

- `generate` reads `.m2s2.json` first, then falls back to `package.json` detection, then prompts
- `new` writes an initial `.m2s2.json` as part of the scaffold

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
- Framework detection reads `package.json` `dependencies` for `@m2s2/ng-lib`, `react-lib`, `vue-lib`
