# CLAUDE.md

Project-specific notes for Claude Code sessions working on `m2s2-cli`.

## Session handoff — e2e hardening (in progress)

Started because scaffolded projects had never actually been verified to build/lint/test/run —
only file-existence was checked. Goal: `m2s2 new` should produce a project that works out of
the box, no manual fixes required. Full history is in the conversation; this is the resume point.

### What's done

- `tests/e2e.rs` rewritten: each scenario now runs `m2s2 build/lint/test`, `m2s2 generate
  component/page(+service)` followed by a rebuild, and a `m2s2 dev` port-smoke-test — not just
  file-existence checks. Expanded to all 21 frontend×backend fullstack combos (31 scenarios
  total). Run with:
  ```
  M2S2_E2E_SKIP_DEV_ASSERT=1 cargo test --test e2e -- --ignored --nocapture --test-threads=1
  ```
  `M2S2_E2E_SKIP_DEV_ASSERT=1` downgrades the dev-smoke-test assertion to a warning — needed
  because this was being run inside a sandboxed agent shell that blocks loopback connections
  between sibling processes (confirmed via manual testing: `m2s2 dev` genuinely works, Vite logs
  "ready" and binds the port, but the sandboxing shell's own `nc`/socket check still can't reach
  it, and `kill -TERM -<pgid>` on the child process group fails with "Operation not permitted").
  On a normal (non-sandboxed) machine, leave the env var unset for a real assertion.
- One starter test added per stack (react/vue/angular/express/fastify/fastapi/flask) so `m2s2
  test` isn't broken on a fresh scaffold (previously none of the 7 non-Go stacks shipped a single
  test file). No new deps added — uses whatever each stack's `package.json`/`requirements.txt`
  already declared.
- Real bugs found and fixed (all confirmed via live e2e runs, not just reasoning):
  1. `go build -o bin/api ./...` fails whenever the module has more than one package (main +
     `internal/*`) — changed to `go build -o bin/api .` (`src/commands/run.rs`).
  2. Python `dev`/`test`/`lint` ran the system `python3`, never the project's own `.venv` —
     switched to `.venv/bin/python3` (`src/commands/run.rs`).
  3. `generate page` (React/Vue) emitted an `index.ts` barrel pointing at `./{{name}}` when the
     actual file is `{{name}}Page.tsx`/`.vue` — added dedicated `page-index.ts.hbs` templates
     (`templates/generate/{react,vue}/page-index.ts.hbs`, wired in `src/commands/generate/page.rs`).
  4. Python lint passed `"check ."` as one argument instead of two (`prepend` → `prepend2` in
     `src/commands/run.rs`) — ruff never actually ran until this was fixed.
  5. fastapi/flask `main.py.hbs`: unsorted import block, flask's unused `request` import, and
     flask's `PORT` env default was an `int` instead of `str` — all ruff findings, now fixed.
     Also added `pythonpath = ["."]` to both `pyproject.toml`s so pytest can `import main`.
  6. **Fullstack version-merge bug** (`src/scaffold/mod.rs`): frontend and backend each resolve
     their own npm version map, but they were merged into one shared map via `.extend()` before
     rendering *both* `apps/web` and `apps/api` — any package both sides declare (chiefly
     `typescript`) had one side's resolved version silently clobber the other's, breaking
     `npm install` in `apps/web` for any frontend + Node-backend fullstack combo. Fixed by giving
     `apps/web` and `apps/api` their own version maps; the merged map is now only used for
     shared/root-level templates (`root`, `fullstack`, `cdk`, `github-actions`).
  7. **npm peer-dependency reconciliation** (`src/npm.rs`, new): live "latest" resolution was
     picking mutually-incompatible versions of related packages.
     - `typescript` vs `typescript-eslint`: capped `typescript` down to the highest version
       `typescript-eslint`'s peer range actually supports (`reconcile_peer_dependency`).
     - `angular-eslint` vs `@angular/cli`: **opposite direction** — `@m2s2/ng-lib`'s own peer
       pin on Angular is the authoritative one (per this CLI's existing design: the m2s2 library
       is the source of truth), so `angular-eslint` gets capped down to match it instead
       (`reconcile_tooling_version`). Get this backwards and you fix the ERESOLVE but break
       `@m2s2/ng-lib`'s own peer requirement instead — that happened once already this session,
       worth double-checking if touched again.
  8. Angular template used `NgNavbarConfig.brandRouterLink`, which doesn't exist on the real
     type (confirmed against the published `@m2s2/ng-lib` `.d.ts`) — real property is
     `brandRouterOutlet` (`templates/angular/src/app/app.component.ts.hbs`).
  9. Express/Fastify `tsconfig.json` missing `rootDir` — TS5011 on newer TypeScript majors.
  10. My own `health.test.ts` for Express imported `AddressInfo` from `node:http` instead of
      `node:net` (`templates/express/src/routes/health.test.ts`).
  11. Angular `jest.config.ts` had a typo, `setupFilesAfterFramework` (not a real Jest option) —
      should be `setupFilesAfterEnv`. Also `jest-environment-jsdom` was missing entirely from
      Angular's devDependencies (required explicitly since Jest 28) — added to
      `package.json.hbs`, the supplemental-package list in `src/scaffold/scaffolder.rs`, and the
      offline-mode placeholder key list in `src/scaffold/mod.rs`.
  12. **Confirmed via live re-run**: fix #11 was correct but insufficient — `setup-jest.ts` also
      imported `jest-preset-angular/setup-jest`, which was removed in jest-preset-angular v17
      (this CLI resolves "latest", and v17 is current). Replaced with the new API:
      `import { setupZoneTestEnv } from 'jest-preset-angular/setup-env/zone'; setupZoneTestEnv();`
      (`templates/angular/setup-jest.ts`). Confirmed fixed in isolation.

### Known environment gotcha (this sandboxed dev machine only)

The sandboxed agent shell can spawn `m2s2 dev` during the e2e fullstack-lifecycle check but,
per the note above, can't reliably kill the process group afterward (`kill -TERM -<pgid>` →
"Operation not permitted"). If a background e2e run gets interrupted mid-suite, the orphaned
`m2s2 dev` process keeps a port bound in a `/tmp/.tmpXXXXXX/...` scaffold dir, and the *next*
e2e run then fails near-instantly (observed: died after starting the very first scenario,
zero useful output). Symptom → fix: `ps aux | grep target/debug/m2s2`, confirm the process's
cwd is a `/tmp/.tmp*` scaffold dir (not something the user is working on), `kill -9` it, retry.
Also note: `cargo test --test e2e -- --ignored ... > logfile 2>&1` run via the harness's
background-bash mechanism was observed to be killed externally (exit code 144) independent of
this issue at least once this session — cause not fully root-caused; if a background e2e run
dies with no clear panic/assertion in the log, suspect the harness rather than the test logic.

### What's NOT done / next steps

1. **Re-run the full e2e matrix once `m2s2-design-system` upstream fixes (item 2 below) are
   published.** Angular and Vue scenarios (16 of 31) are both currently blocked on upstream
   `@m2s2/*` package bugs, not on anything fixable in this repo — see item 2. The other 15
   scenarios (React frontend, all 7 backends, React fullstack ×7) were being re-verified when
   this handoff was written; check their result before assuming they're clean.
2. **Three confirmed upstream packaging bugs in `m2s2-design-system`, all blocking `m2s2 test`
   for the affected framework's scaffolds.** User's decision (2026-07-30): **user will fix these
   upstream themselves** rather than have `m2s2-cli` work around them in templates. Re-run the
   full e2e matrix once new versions are published, since the fix is external to this repo.
   - `@m2s2/vue-lib`: `package.json` `types`/`exports.types` points at `dist/index.d.ts`, but the
     actual `.d.ts` files in the published tarball live at `dist/vue-lib/src/index.d.ts` instead.
     Breaks `vue-tsc` on every Vue scaffold (8/31 e2e scenarios). Local checkout referenced
     previously at `/Users/mgmaster24/projects/m2s2-design-system` (path may differ on this
     machine — confirm before assuming it's there).
   - `@m2s2/utils` and `@m2s2/models`: both published ESM-only — `package.json` `exports` map has
     only an `"import"` condition, no `"require"`/`"main"` fallback, and their JS entry files
     aren't named `.mjs` either. `@m2s2/ng-lib`'s own bundle (`fesm2022/*.mjs`) imports both, and
     jest-preset-angular's default CJS preset can't resolve either through Jest's CJS resolver —
     confirmed via direct reproduction (`Cannot find module '@m2s2/utils'` then, after working
     around that one, `Cannot find module '@m2s2/models'`, both from inside
     `node_modules/@m2s2/ng-lib/fesm2022/m2s2-ng-lib.mjs`). This breaks **100% of Angular
     scaffolds' `m2s2 test`** (8/31 e2e scenarios) — not an edge case. A local jest.config
     `moduleNameMapper`/`transformIgnorePatterns` workaround was verified to work package-by-
     package but was explicitly rejected in favor of the upstream fix.
3. Coverage still not exercised in the e2e suite: `--auth yes` / `--billing yes` (only checked
   for file-existence in the fast/offline unit tests, never actually installed+built+linted), and
   CDK output isn't `cdk synth`-validated.
4. Nothing has been pushed to a remote. Commits made this session are local only.

## `publish` command (shipped 2026-07-30, refactored same day)

New feature, unrelated to the scaffolding domain everything else in this CLI covers: `m2s2
publish <file.md>` publishes a Markdown article (YAML frontmatter) to Dev.to, Hashnode, and/or
a generic blog "platform" target. Went through several rounds of follow-up refactoring the same
day it shipped — this section describes the *current* shape, not the original one; don't trust
old commit messages over this.

- **Layout**: `src/publish/{article,config,target,target_kind}.rs` + `src/publish/targets/
  {devto,hashnode,platform}.rs`. Command glue in `src/commands/publish.rs`.
- **`TargetKind`** (`target_kind.rs`): `Devto`/`Hashnode`/`Platform` enum, derives
  `clap::ValueEnum` + `serde::Deserialize` + `Serialize` — used for both the frontmatter
  `publish:` list and `--to`, so an invalid name is a parse-time error (clap/serde generate the
  "possible values" message) rather than a runtime string-match failure. Matches the existing
  `Frontend`/`Runtime`/etc. pattern in `src/types.rs`.
- **`Target`** (`target.rs`): plain enum (`Devto(DevTo)`/`Hashnode(Hashnode)`/
  `Platform(Platform)`), not `Box<dyn Trait>` — deliberately no trait object here. The set of
  targets is closed and known at compile time (a new one always means writing a match arm in
  `targets::mod::build_one` regardless), so dyn dispatch bought indirection without buying
  flexibility. Each concrete target has an ordinary inherent `async fn publish(...)`; there is
  no `PublishTarget` trait and no `async-trait` dependency — both were only needed for the
  `dyn Trait` case and got removed when it did.
- **`HttpTarget`** (`target.rs`): shared `{client, base_url}` pair every target embeds. The
  `reqwest::Client` is built once in `targets::build_targets` and `.clone()`d (cheap — it's
  `Arc`-backed) into each target, instead of every target building its own connection
  pool/TLS cache.
- **Article format**: Markdown + YAML frontmatter (`title`, `date`, `summary`, `tags`, `slug`
  optional/derived from filename, `excerpt`/`cover_image`/`canonical_url` optional, `publish:
  [...]` target list, overridable by `--to`). Deliberately matches what `m2s2-platform`'s own
  admin blog editor already exports as Markdown (confirmed by reading
  `apps/web/src/app/admin/blog-edit/admin-blog-edit.component.ts` in that repo) — not a new,
  competing format. `Article` derives `Serialize` (used by the platform target's `body_command`
  hook, below).
- **Config**: `.m2s2-publish.toml` in the CWD (flat dotfile, matching `.m2s2.json`'s existing
  convention), `[devto]`/`[hashnode]`/`[platform]` sections. Contains secrets — not gitignored by
  *this* repo since the file lives in whatever directory the *user* runs `m2s2 publish` from
  (their own blog-content repo), not in a scaffolded project.
- **The `platform` target is deliberately generic, not m2s2-specific** — user feedback
  (2026-07-30): this is a public tool, and baking the maintainer's own site's assumptions into
  the shipped binary is wrong twice over — it leaks implementation details of their site to
  every reader of a public repo, *and* it means the tool is useless to anyone whose blog API
  differs. Originally named/typed `M2s2`/`M2s2Config` (confusing alongside the unrelated
  `M2S2Config` in `src/config.rs`, the scaffolding config) — renamed throughout to
  `Platform`/`PlatformConfig`. Concretely, nothing about *any* specific deployment is hardcoded:
  - `endpoint` (base URL) and `token` (bearer auth) — always were config, not hardcoded.
  - `path` — appended to `endpoint` for both create (`POST <path>`) and update
    (`PUT <path>?slug=...`). **Config field, defaults to `/admin/blog`** (that default is the
    maintainer's own convention — everyone else overrides it).
  - `body_command` — **the request body itself is now a hook, not fixed Rust field mapping.**
    If set, the article (plus `update: true/false`) is piped to this command as JSON on stdin,
    and whatever JSON it prints on stdout is sent **verbatim** as the request body (no merging
    with the built-in mapping). Runs through a shell (`sh -c` / `cmd /C` on Windows), so it can
    be a script path or a full command line with args — same shape as git hooks / husky /
    kubectl credential plugins, deliberately *not* a WASM runtime or dylib-loading plugin
    system: with zero second targets in sight, that would be solving a hypothetical, not
    building a plugin ABI to carry forever for one real user.
  - If `body_command` is unset, falls back to the original fixed field mapping (`slug`, `title`,
    `date`, `summary`, `excerpt`, `tags`, `coverImage`, `content`) — this mapping is still what
    `m2s2-platform` itself actually needs (verified against its real
    `apps/api/dashboard/handlers/blog.go`), it's just no longer the *only* option.
- **Platform auth is a static bearer token pasted into config, manually refreshed** — deliberate
  scope cut, not yet revisited: the maintainer's own platform's real login (`apps/web/src/
  environments/environment.prod.ts`) is Amplify/Cognito with SRP + optional TOTP MFA, which is
  substantial scope beyond what the CLI needed. A `m2s2 login` companion command doing the full
  Cognito dance would be the natural fast-follow if manual token refresh gets annoying — but
  that's specific to whoever's `token` config points at a Cognito-backed API; it's not something
  the generic `platform` target should assume either.
- **Dev.to**: verified against current Forem API docs — `POST https://dev.to/api/articles`,
  `api-key` header, `{"article": {...}}` body, comma-joined tags (max 4).
- **Hashnode**: verified against current docs/search (their API changed **May 2026** — legacy
  `api.hashnode.com` is discontinued, new endpoint is `https://gql.hashnode.com`, and **publish
  access now requires a paid Hashnode Pro subscription** — user confirmed they have it).
  `publishPost(input: PublishPostInput!)` GraphQL mutation, `Authorization: Bearer <PAT>`, tags
  as `{name, slug}` objects. GraphQL errors surface in the response body even on HTTP 200 — the
  target explicitly checks the `errors` field, not just HTTP status.
- **Both external targets (Dev.to, Hashnode) reject `--update`** with a clear error (scope cut —
  only the platform target's create-vs-update distinction was actually specified; no `body_command`-
  style escape hatch for them either, since there's been no request for one).
- **Tests**: `httpmock` (dev-dependency) mocks the HTTP layer per target — request shape
  (headers, body, method, query params) and response parsing are asserted against a real local
  server, not just reasoned about. The `body_command` hook tests spawn real `sh -c` subprocesses
  (`echo`, `cat`, a deliberately failing command) rather than mocking process execution. 43 unit
  tests total, all passing; clippy clean. Also manually smoke-tested the built binary once
  against dev.to's real API with a fake key (clean 403 — confirms the request path and CLI
  error/exit-code handling end to end, not just the mocked path).
- **Not done**: no integration/e2e test added to `tests/e2e.rs` (would need each target's real
  credentials, so out of scope for the offline/CI-friendly e2e suite); `--auth yes`/`--billing
  yes` and CDK-synth coverage gaps noted above are still open, unrelated to this feature.
  `body_command` only exists for the `platform` target, not Dev.to/Hashnode — no request for
  that yet either.
- **`cover_image` handling** (`src/publish/cover_image.rs`, new module, shared by all three
  targets): the frontmatter value is resolved into either `CoverImage::Url` (starts with
  `http://`/`https://`, passed straight through) or `CoverImage::Local` (anything else — read
  relative to `Article::base_dir`, i.e. the directory containing the `.md` file, then base64-
  encoded eagerly so a bad/missing local file fails before any network call happens).
  - **Dev.to and Hashnode are verified URL-only** — checked their actual docs, not assumed: Dev.to's
    Forem API has *no image upload endpoint at all* (`main_image` is documented as "Absolute URL
    of the cover image"); Hashnode's `publishPost` mutation only takes
    `coverImageOptions.coverImageURL` — note the nested-object shape, not a flat `coverImage`
    field like `platform` uses. A local path for either target is a clear error
    (`cover_image::local_path_not_supported_error`) telling the user to host it themselves —
    there is no upload fallback to build for these two, because their APIs genuinely don't
    support one.
  - **`platform` uploads local files** using the same base64 mechanism `blog.go`'s
    `maybeUploadCover` already expects (`coverImageFilename`/`coverImageData`/
    `coverImageContentType`) — this was already read and understood when the platform target was
    first built, just not wired up until now.
- **Investigated (2026-07-30) whether the CLI should auto-host local images for Dev.to/Hashnode
  via presigned URLs, per user suggestion — concluded not yet, and why**: checked
  `m2s2-platform` for an existing presigned-URL/generic-upload endpoint first. Found
  `GET /admin/resume/upload-url` (`apps/api/dashboard/handlers/resume.go`,
  `AdminResumeUploadURL`) — but it's hardcoded to one fixed S3 key/bucket for *the* resume PDF,
  not a generic "give me a presigned URL for this filename" endpoint. Nothing equivalent exists
  for blog images. Also checked whether the create/update blog response could hand back the
  uploaded image's final URL for reuse elsewhere (e.g. publish to `platform` first, then reuse
  its now-hosted image URL for Dev.to/Hashnode in the same run) — it can't: both
  `AdminCreateBlogPost`/`AdminUpdateBlogPost` return only `{"message": "..."}`, never the
  computed `CoverImage` path, and the public site's base URL (`siteBaseURL` in blog.go) is a Go
  constant, not something the API exposes — so the CLI has no reliable way to derive the final
  URL itself either. **Conclusion: this needs new server-side work in `m2s2-platform` (a real
  generic presign/upload endpoint, or returning the image URL in the create/update response)
  before the CLI can build on it — it's a two-repo feature, not something `m2s2-cli` can finish
  alone.** Not started. If picked up later: decide there first whether to go with presigned S3
  PUT URLs (client uploads directly, platform never sees the bytes) or keep the existing
  base64-through-Lambda pattern generalized into a standalone endpoint — the two have different
  size limits and latency characteristics worth weighing before choosing.

### Post-review hardening (2026-07-31)

User did their own review of the whole `publish` feature and flagged several real gaps, all
fixed same day: `Target::preflight()` added (validates every selected target — `cover_image`
compatibility *and* `--update` support — before any of them makes a network request, since
Dev.to/Hashnode have no update support and a partial publish followed by a retry would create
duplicates there); malformed/incomplete success responses in `devto.rs`/`hashnode.rs` are now
hard errors instead of silently reporting "published, no URL"; Dev.to's documented
`Accept: application/vnd.forem.api-v1+json` header added; `M2S2_PUBLISH_*` env-var credential
fallback added to `PublishConfig::load` (only fills in a *wholly absent* `[section]`, TOML
always wins if present); full `m2s2 publish` section added to README.

**Caught after the first pass, worth remembering**: `preflight()` initially only checked
`cover_image` — missed that `--update` support is *also* checked inside each target's `publish`
before any network call (`if update { bail!(...) }` at the top of `devto.rs`/`hashnode.rs`), the
exact same "validation that fires too late" bug class preflight exists to prevent. Fixed by
extracting `DevTo::check_update_supported`/`Hashnode::check_update_supported` (`pub(crate)`),
shared between `publish()` and `Target::preflight()` so the two can't drift apart again. Grepped
every `bail!`/`Err(` in all three target files to confirm nothing else pre-network-call is still
uncovered — everything else is response handling, after a request already went out, which isn't
preflight's job.

Deferred as its own follow-up (design already agreed, not started): a local
`.m2s2-publish-state.json`-style file (slug → target ID/URL/status), gitignored, enabling
Dev.to/Hashnode update-by-ID (both APIs confirmed to support it — `PUT /api/articles/{id}` for
Dev.to, `updatePost(input: UpdatePostInput!)` for Hashnode) and safe retries after a partial
failure. Also deferred: retry logic for transient (network/5xx) failures specifically.

### `prepare`/`execute` split + `--preflight-only` (2026-07-31)

User added three design docs under `docs/` (`publishing-platform-evaluation.md`,
`api-verification-preflight.md`, `ai-social-content-architecture.md`) describing a much larger
expansion: contract-pinned request validation (vendored OpenAPI/GraphQL schemas + scheduled
CI diff jobs), read-only remote verification, a `PreflightReport`/`--format json`, and a
completely separate LLM-backed `m2s2 content generate` feature for LinkedIn/X drafts. Too much
for one pass — implemented only `api-verification-preflight.md`'s own "Phase 1: eliminate
predictable partial publishes," the smallest slice that's actually complete and shippable on
its own. Everything else in the three docs is still just a doc — see them directly for the full
proposals if picking this back up.

**What shipped**: `Target::preflight()` → `Target::prepare()`, returning a `PreparedTarget`
enum (mirrors `Target` itself — `Devto`/`Hashnode`/`Platform` variants, no trait object, same
rationale as `Target`'s own doc comment) instead of `Result<()>`. `Target::publish()` →
`Target::execute(prepared: PreparedTarget)`. Each concrete target
(`devto.rs`/`hashnode.rs`/`platform.rs`) got the same split: a `PreparedRequest` struct (owned
data, deliberately no lifetime parameter — see the doc comment on `devto::PreparedRequest` for
why), `prepare()` builds it (local-only, including — for `Platform` — running `body_command`,
which must happen exactly once since it's arbitrary user-configured code), `execute()` sends it.
`commands/publish.rs` now does prepare-all-then-execute-all instead of two separately-computed
loops; added `--preflight-only`, which stops after a successful prepare-all pass and prints a
per-target summary — no `--offline` flag or JSON report yet, both explicitly deferred to when
Phase 2 (remote verification) actually needs them; nothing to skip offline yet since `prepare`
never touches the network.

**Verified live, not just via tests**: built a throwaway article + `.m2s2-publish.toml` with a
deliberately wrong Dev.to API key. `--preflight-only` succeeded (proves no network call — a real
attempt would've hit Dev.to's real API and failed on the bad key); running the same command
*without* the flag did reach the real API and correctly got a 403. All 67 unit tests pass
(rewrote every test that called the old single-step `publish()` into `prepare()` then
`execute()`), clippy/fmt clean, pre-commit hook exits 0.

Explicitly deferred, with one-line reasons already captured in the plan file at the time (check
`/home/mgmaster/.claude/plans/` if it's still there, otherwise this paragraph is the summary):
contract pinning + remote verification (Phase 2/3 of the preflight doc — real new dependencies,
a new CI job, not needed for this phase to stand on its own); publication-state + ID-based
updates (inert without a structured remote ID, and today's `PublishOutcome.message` is a display
string — bundling means widening that interface for a feature this pass doesn't need); the
entire AI content-generation feature (self-contained, shares nothing with `publish` but
`Article` — its own plan/session; worth reusing `Article`/`parse_article`, the `cover_image.rs`
Url-vs-Local/`base_dir`-relative pattern, and `config.rs`'s "env fills in only a wholly-absent
section" fallback rule rather than reinventing any of them).
