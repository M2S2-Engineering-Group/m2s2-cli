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

## `publish` command (shipped 2026-07-30)

New feature, unrelated to the scaffolding domain everything else in this CLI covers: `m2s2
publish <file.md>` publishes a Markdown article (YAML frontmatter) to Dev.to, Hashnode, and/or
the user's own m2s2-platform blog. Built as a `PublishTarget` trait (`src/publish/target.rs`) so
adding a new connector is "implement the trait + one match arm in
`src/publish/targets/mod.rs::build_one`" — explicitly requested by the user for extensibility,
not a speculative abstraction.

- **Layout**: `src/publish/{article,config,target}.rs` + `src/publish/targets/{devto,hashnode,
  m2s2}.rs`. Command glue in `src/commands/publish.rs`.
- **Article format**: Markdown + YAML frontmatter (`title`, `date`, `summary`, `tags`, `slug`
  optional/derived from filename, `excerpt`/`cover_image`/`canonical_url` optional, `publish:
  [...]` target list, overridable by `--to`). Deliberately matches what `m2s2-platform`'s own
  admin blog editor already exports as Markdown (confirmed by reading
  `apps/web/src/app/admin/blog-edit/admin-blog-edit.component.ts` in that repo) — not a new,
  competing format.
- **Config**: `.m2s2-publish.toml` in the CWD (flat dotfile, matching `.m2s2.json`'s existing
  convention rather than a `.m2s2/` subdirectory), `[devto]`/`[hashnode]`/`[m2s2]` sections.
  Contains secrets — not gitignored by this repo since it lives in whatever directory the *user*
  runs `m2s2 publish` from (their own blog-content repo), not in a scaffolded project.
- **m2s2 target contract** verified by reading `m2s2-platform`'s actual source
  (`apps/api/dashboard/handlers/blog.go`), not guessed: `POST /admin/blog` (create, 409 if slug
  exists) / `PUT /admin/blog?slug=` (update), Cognito JWT bearer auth requiring the `admin`
  Cognito group claim. **Auth is a static bearer token pasted into config, manually refreshed**
  — deliberate v1 scope cut: the platform's real login (`apps/web/src/environments/
  environment.prod.ts`) is Amplify/Cognito with SRP + optional TOTP MFA, which is substantial
  scope beyond what the CLI needed today. A `m2s2 login` companion command doing the full
  Cognito dance would be the natural fast-follow if manual token refresh gets annoying.
- **Dev.to**: verified against current Forem API docs — `POST https://dev.to/api/articles`,
  `api-key` header, `{"article": {...}}` body, comma-joined tags (max 4).
- **Hashnode**: verified against current docs/search (their API changed **May 2026** — legacy
  `api.hashnode.com` is discontinued, new endpoint is `https://gql.hashnode.com`, and **publish
  access now requires a paid Hashnode Pro subscription** — user confirmed they have it).
  `publishPost(input: PublishPostInput!)` GraphQL mutation, `Authorization: Bearer <PAT>`, tags
  as `{name, slug}` objects. GraphQL errors surface in the response body even on HTTP 200 — the
  target explicitly checks the `errors` field, not just HTTP status.
- **Both external targets reject `--update`** with a clear error (v1 scope cut — only the m2s2
  target's create-vs-update distinction was actually specified).
- **Tests**: `httpmock` (new dev-dependency) mocks the HTTP layer per target — request shape
  (headers, body, method, query params) and response parsing (including the Hashnode
  errors-on-200 case and the m2s2 409-conflict case) are asserted against a real local server,
  not just reasoned about. 16 new unit tests, all passing; full suite (39) and clippy clean.
  Manually smoke-tested the built binary too (`m2s2 publish post.md --to devto` with a fake key
  against the *real* dev.to API — got a clean 403, confirming the request actually reaches
  their API and the CLI's error/exit-code path works end to end).
- **Not done**: no integration/e2e test added to `tests/e2e.rs` (would need each target's real
  credentials, so out of scope for the offline/CI-friendly e2e suite); `--auth yes`/`--billing
  yes` and CDK-synth coverage gaps noted above are still open, unrelated to this feature.
