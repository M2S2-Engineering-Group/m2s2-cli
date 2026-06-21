# m2s2-cli

[![GitHub Sponsors](https://img.shields.io/github/sponsors/mgmaster24?style=flat&logo=githubsponsors&label=Sponsor)](https://github.com/sponsors/mgmaster24)

The official CLI for scaffolding and working with [M²S²](https://github.com/M2S2-Engineering-Group) design system projects. Create new frontend (React, Angular, Vue), backend (Go, Node, Python), or fullstack projects pre-wired with M²S² components, generate components and pages, and keep your installation up to date — all from the terminal.

## Table of Contents

- [Installation](#installation)
- [Commands](#commands)
  - [new](#m2s2-new)
  - [generate component](#m2s2-generate-component)
  - [generate page](#m2s2-generate-page)
  - [generate service](#m2s2-generate-service)
  - [upgrade](#m2s2-upgrade)
  - [completions](#m2s2-completions)
- [Building from Source](#building-from-source)
- [Testing](#testing)
- [Project Structure](#project-structure)
- [Workflows](#workflows)
- [Contributing](#contributing)

---

## Installation

### Shell installer (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/M2S2-Engineering-Group/m2s2-cli/releases/latest/download/m2s2-cli-installer.sh | sh
```

### PowerShell (Windows)

```powershell
irm https://github.com/M2S2-Engineering-Group/m2s2-cli/releases/latest/download/m2s2-cli-installer.ps1 | iex
```

### npm / npx

```bash
# Install globally
npm install -g @m2s2/cli

# Or run without installing
npx @m2s2/cli new my-app
```

### Cargo

```bash
cargo install m2s2-cli
```

### Verify installation

```bash
m2s2 --version
```

---

## Commands

### `m2s2 new`

Scaffold a new project pre-configured with the M²S² design system.

```bash
m2s2 new <name> [OPTIONS]
```

**Arguments**

| Argument | Description |
|----------|-------------|
| `<name>` | Project directory name |

**Options**

| Flag | Values | Description |
|------|--------|-------------|
| `--project-type` | `frontend` `backend` `fullstack` | Project type. Prompted interactively if omitted. |
| `--framework` | `react` `angular` `vue` | Frontend framework. Required for frontend and fullstack projects. |
| `--runtime` | `go` `node` `python` | Backend runtime. Required for backend and fullstack projects. |
| `--api-framework` | `gin` `echo` `fiber` `express` `fastify` `fastapi` `flask` | API framework. Infers `--runtime` if omitted. |
| `--skip-install` | | Skip `npm install` / `go mod tidy` / `pip install` after scaffolding. |
| `--offline` | | Skip npm version resolution (uses placeholder versions). Intended for testing. |

**Examples**

```bash
# Interactive — prompts for all choices
m2s2 new my-app

# Frontend
m2s2 new my-app --project-type frontend --framework react
m2s2 new my-app --project-type frontend --framework angular
m2s2 new my-app --project-type frontend --framework vue

# Backend — Go
m2s2 new my-api --project-type backend --runtime go --api-framework gin
m2s2 new my-api --project-type backend --runtime go --api-framework echo
m2s2 new my-api --project-type backend --runtime go --api-framework fiber

# Backend — Node
m2s2 new my-api --project-type backend --runtime node --api-framework express
m2s2 new my-api --project-type backend --runtime node --api-framework fastify

# Backend — Python (creates a .venv and runs pip install)
m2s2 new my-api --project-type backend --runtime python --api-framework fastapi
m2s2 new my-api --project-type backend --runtime python --api-framework flask

# Fullstack
m2s2 new my-app --project-type fullstack --framework react --runtime go --api-framework gin
m2s2 new my-app --project-type fullstack --framework vue --runtime python --api-framework fastapi

# Skip install (useful in CI)
m2s2 new my-app --project-type frontend --framework react --skip-install
```

**What gets generated**

*React*
```
my-app/
├── src/
│   ├── main.tsx        # BrowserRouter + M2S2Provider
│   ├── App.tsx         # Navbar + Footer wired with M²S² configs
│   └── App.scss
├── index.html
├── package.json        # @m2s2/react-lib, @m2s2/tokens, Vite, TypeScript
├── vite.config.ts
├── tsconfig.json
└── .gitignore
```

*Angular*
```
my-app/
├── src/
│   ├── main.ts
│   ├── styles.scss
│   ├── index.html
│   └── app/
│       ├── app.component.ts    # Standalone, imports NavbarComponent + FooterComponent
│       ├── app.component.html
│       ├── app.component.scss
│       ├── app.routes.ts
│       └── app.config.ts
├── package.json                # @m2s2/ng-lib, @m2s2/tokens, Angular 21
├── angular.json
├── tsconfig.json
├── tsconfig.app.json
└── .gitignore
```

*Vue*
```
my-app/
├── src/
│   ├── main.ts         # createApp with @m2s2/vue-lib styles
│   ├── App.vue         # Navbar + Footer wired with M²S² configs
│   └── App.scss
├── index.html
├── package.json        # @m2s2/vue-lib, @m2s2/tokens, Vite, TypeScript
├── vite.config.ts
├── tsconfig.json
└── .gitignore
```

---

### `m2s2 generate component`

Scaffold a new component inside an existing M²S² project.

```bash
m2s2 generate component <name> [OPTIONS]
```

Run this command from your project root. The framework is detected automatically by reading `package.json` — no flag needed if your project was created with `m2s2 new`.

**Arguments**

| Argument | Description |
|----------|-------------|
| `<name>` | Component name. Accepts any casing — `MyCard`, `my-card`, and `myCard` all produce the same output. |

**Options**

| Flag | Description |
|------|-------------|
| `--framework <react\|angular\|vue>` | Override framework detection. |
| `--path <dir>` | Override the output directory. A subdirectory named after the component is always created inside `<dir>`. |

**Examples**

```bash
# Auto-detect framework from package.json
m2s2 generate component HeroSection

# Kebab-case input — same result
m2s2 generate component hero-section

# Override output directory
m2s2 generate component HeroSection --path src/features

# Override framework
m2s2 generate component HeroSection --framework vue
```

**What gets generated**

*React* — written to `src/components/<Name>/`
```
src/components/HeroSection/
├── HeroSection.tsx     # Typed props interface, BEM className application
├── HeroSection.scss    # Scoped class stub (.hero-section)
└── index.ts            # Barrel re-export
```

*Angular* — written to `src/app/components/<name>/`
```
src/app/components/hero-section/
├── hero-section.component.ts    # Standalone component, app-hero-section selector
├── hero-section.component.html  # BEM wrapper div
└── hero-section.component.scss  # Scoped class stub (.hero-section)
```

*Vue* — written to `src/components/<Name>/`
```
src/components/HeroSection/
├── HeroSection.vue     # <script setup> SFC with slot and scoped SCSS
├── HeroSection.scss    # Scoped class stub (.hero-section)
└── index.ts            # Barrel re-export
```

---

### `m2s2 generate page`

Scaffold a new route page inside an existing M²S² project.

```bash
m2s2 generate page <name> [OPTIONS]
```

Run this from your project root. Framework is auto-detected from `package.json` or `.m2s2.json`.

**Arguments**

| Argument | Description |
|----------|-------------|
| `<name>` | Page name. Accepts any casing — `UserProfile`, `user-profile`, and `userProfile` all produce the same output. |

**Options**

| Flag | Description |
|------|-------------|
| `--framework <react\|angular\|vue>` | Override framework detection. |
| `--path <dir>` | Override the output directory. |

**Examples**

```bash
m2s2 generate page UserProfile
m2s2 generate page user-profile --framework angular
m2s2 generate page Dashboard --path src/views
```

**What gets generated**

*React* — written to `src/pages/<Name>/`
```
src/pages/UserProfile/
├── UserProfilePage.tsx   # Typed props, BEM className
├── UserProfilePage.scss  # Scoped class stub
└── index.ts              # Barrel re-export
```

*Angular* — written to `src/app/pages/<name>/`
```
src/app/pages/user-profile/
├── user-profile.component.ts    # Standalone, OnPush, app-user-profile selector
├── user-profile.component.html
└── user-profile.component.scss
```

A lazy-load route snippet is printed to the terminal after generation:
```typescript
{ path: 'user-profile', loadComponent: () => import('./pages/user-profile/user-profile.component').then(m => m.UserProfilePageComponent) }
```

*Vue* — written to `src/pages/<Name>/`
```
src/pages/UserProfile/
├── UserProfilePage.vue   # <script setup> SFC with scoped SCSS
└── index.ts              # Barrel re-export
```

---

### `m2s2 generate service`

Scaffold an injectable Angular service inside an existing M²S² project.

```bash
m2s2 generate service <name> [OPTIONS]
```

> Only supported for Angular projects. React and Vue projects receive a helpful error message.

**Arguments**

| Argument | Description |
|----------|-------------|
| `<name>` | Service name (without the `Service` suffix). |

**Options**

| Flag | Description |
|------|-------------|
| `--path <dir>` | Override the output directory. Defaults to `src/app/services/`. |

**Examples**

```bash
m2s2 generate service Auth
m2s2 generate service user-data
m2s2 generate service Analytics --path src/app/core/services
```

**What gets generated** — written to `src/app/services/<name>/`
```
src/app/services/auth/
└── auth.service.ts   # @Injectable({ providedIn: 'root' }) AuthService
```

---

### `m2s2 completions`

Install shell completions for `m2s2`. Auto-detects your shell from `$SHELL` and writes a completion script to your home directory, then patches your shell's rc file to source it automatically.

```bash
m2s2 completions [shell]
```

**Arguments**

| Argument | Description |
|----------|-------------|
| `[shell]` | Shell to generate completions for. Auto-detected from `$SHELL` if omitted. One of: `bash`, `zsh`, `fish`, `elvish`, `powershell`. |

**Examples**

```bash
# Auto-detect shell and install
m2s2 completions

# Explicit shell
m2s2 completions zsh
```

After running, reload your shell or source your rc file:

```bash
source ~/.zshrc   # zsh
source ~/.bashrc  # bash
# fish sources completions automatically — no reload needed
```

---

### `m2s2 upgrade`

Check for and install a newer version of the CLI.

```bash
m2s2 upgrade [OPTIONS]
```

**Options**

| Flag | Description |
|------|-------------|
| `--check` | Print available version without installing. |

**Examples**

```bash
# Check if an update is available
m2s2 upgrade --check

# Download and install the latest version
m2s2 upgrade
```

The upgrade command hits the GitHub Releases API, compares the latest release tag against the running binary version, and — if a newer version exists — runs the official installer script for your platform. Restart your terminal after upgrading.

---

## Building from Source

**Prerequisites:** Rust 1.85+ (edition 2024), Node.js 18+ for running scaffolded projects.

```bash
git clone https://github.com/M2S2-Engineering-Group/m2s2-cli.git
cd m2s2-cli

# Debug build
cargo build

# Run directly
cargo run -- new my-app --framework react --skip-install

# Optimised release build (matches the distributed binary)
cargo build --profile dist
```

The binary is written to `target/debug/m2s2` (debug) or `target/dist/m2s2` (dist profile).

### Templates

All scaffold and generate templates live under `templates/` and are embedded into the binary at compile time via [`rust-embed`](https://github.com/pyros2097/rust-embed). Changes to template files require a rebuild to take effect.

```
templates/
├── react/               # m2s2 new --framework react
├── angular/             # m2s2 new --framework angular
├── vue/                 # m2s2 new --framework vue
└── generate/
    ├── react/           # m2s2 generate component/page (React)
    ├── angular/         # m2s2 generate component/page/service (Angular)
    └── vue/             # m2s2 generate component/page (Vue)
```

---

## Testing

The test suite has two tiers.

### Unit / integration tests (fast, no network)

```bash
cargo test
```

Covers all scaffold and generate paths using temp directories and an `--offline`
flag that skips npm version resolution. Runs in under a second. Also includes
`cargo clippy` and `cargo fmt` checks in CI.

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### End-to-end tests (network required)

E2e tests scaffold real projects, resolve live package versions, and run the
full install step (`npm install`, `go mod tidy`, `python3 -m venv` + `pip
install`). They are marked `#[ignore]` and never run during normal CI.

```bash
# Run the full e2e suite
cargo e2e

# Run a single scenario
cargo test --test e2e new_react_frontend_e2e -- --ignored --nocapture
```

Each test asserts on install artifacts to confirm the toolchain ran:

| Scenario | Artifact checked |
|----------|-----------------|
| React / Angular / Vue frontend | `node_modules/` |
| Go backend (gin / echo / fiber) | `go.sum` |
| Node backend (express / fastify) | `node_modules/` |
| Python backend (fastapi / flask) | `.venv/` + `.venv/bin/pip` |
| Fullstack | web `node_modules/` + api artifact |

Temp directories are cleaned up automatically after each test.

---

## Project Structure

```
src/
├── main.rs                        # CLI entry point, command routing
├── utils.rs                       # Shared case conversion (to_pascal_case, to_kebab_case)
├── config.rs                      # .m2s2.json read/write, framework detection/resolution
├── scaffold/
│   └── mod.rs                     # Template engine (rust-embed + Handlebars), write_files helper
└── commands/
    ├── mod.rs
    ├── new.rs                     # m2s2 new
    ├── upgrade.rs                 # m2s2 upgrade
    └── generate/
        ├── mod.rs                 # m2s2 generate (subcommand router)
        ├── component.rs          # m2s2 generate component
        ├── page.rs               # m2s2 generate page
        └── service.rs            # m2s2 generate service
```

---

## Workflows

### CI (`ci.yml`)

Runs on every push to `main` and every pull request.

| Job | Description |
|-----|-------------|
| `build` | Compiles the crate for macOS (arm64), Linux (x86_64), and Windows (x86_64) in a matrix. |
| `test` | Runs `cargo test` on Linux. |
| `clippy` | Runs `cargo clippy -- -D warnings`. |
| `fmt` | Runs `cargo fmt --check`. |

### Release (`release-plz.yml` + `auto-merge-release.yml` + `release-tag.yml` + `release.yml`)

Releases are fully automated from conventional commits — no manual tagging or PR review required.

1. **CI passes on `main`** — `release-plz.yml` triggers only after the `CI` workflow succeeds.
2. **Release PR opened** — `release-plz release-pr` reads conventional commit history, bumps `Cargo.toml`, updates `CHANGELOG.md`, and opens a PR (e.g. `chore: release v0.1.5`). Only `feat`, `fix`, and `perf` commits trigger a bump.
3. **PR auto-approved and merged** — `auto-merge-release.yml` detects the `release-plz-*` branch, approves the PR using the GitHub App token, and immediately squash-merges it.
4. **Tag pushed** — `release-tag.yml` fires on the merge commit, reads the version from `Cargo.toml`, and pushes the `vX.Y.Z` tag if it doesn't already exist.
5. **`release.yml` (cargo-dist) triggers** — builds platform binaries in parallel:
   - `aarch64-apple-darwin`
   - `x86_64-apple-darwin`
   - `aarch64-unknown-linux-gnu`
   - `x86_64-unknown-linux-gnu`
   - `x86_64-pc-windows-msvc`
6. **GitHub Release created** — all binaries, checksums, shell installer, and PowerShell installer are attached.
7. **`publish-npm.yml` triggers** — publishes `@m2s2/cli` to npm.
8. **`publish-crates.yml` triggers** — publishes `m2s2-cli` to crates.io.

### Template Sync (`template-sync.yml`)

Keeps scaffold template dependencies in sync with the latest published M²S² library versions.

Triggers:
- **Weekly** — every Monday at 08:00 UTC.
- **`repository_dispatch`** — fired automatically by the design system CI whenever any `@m2s2` library publishes a new release.

When a version change is detected, the workflow opens a pull request updating the pinned package versions in `templates/react/package.json.hbs`, `templates/angular/package.json.hbs`, and `templates/vue/package.json.hbs`. Merging that PR flows through the normal release pipeline.

### Required Repository Secrets

| Secret | Used By | Description |
|--------|---------|-------------|
| `APP_ID` | `release-plz.yml`, `auto-merge-release.yml`, `release-tag.yml`, `release.yml` | GitHub App ID used to open, approve, merge release PRs, and push tags. |
| `APP_PRIVATE_KEY` | `release-plz.yml`, `auto-merge-release.yml`, `release-tag.yml`, `release.yml` | GitHub App private key. |
| `NPM_TOKEN` | `publish-npm.yml` | npm access token with publish rights to the `@m2s2` scope. |
| `CARGO_REGISTRY_TOKEN` | `publish-crates.yml` | crates.io API token for publishing `m2s2-cli`. |

---

## Contributing

Commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) specification — this is what release-plz uses to determine version bumps and generate the CHANGELOG.

| Prefix | Version bump |
|--------|-------------|
| `fix:` | Patch |
| `feat:` | Minor |
| `feat!:` / `BREAKING CHANGE` | Major |
| `chore:`, `docs:`, `refactor:` | No release |

```bash
# Good examples
git commit -m "feat: add m2s2 init command"
git commit -m "fix: detect camelCase component names correctly"
git commit -m "chore: update Angular template to v22"
```
