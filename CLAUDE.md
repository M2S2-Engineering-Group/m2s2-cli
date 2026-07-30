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

### What's NOT done / next steps

1. **Re-run the full e2e matrix** to confirm fixes #11 (Angular jest config) actually land —
   they were applied but not yet re-verified via a live run before this handoff. Last confirmed
   state: 15/31 passing, with all 8 remaining Angular failures at the `m2s2 test` step (jest
   config), and #11 should fix that. Command is above.
2. **`@m2s2/vue-lib` ships without usable type declarations** — confirmed via direct tarball
   inspection: the published package's `package.json` `types`/`exports.types` field points at
   `dist/index.d.ts`, but the actual `.d.ts` files in the tarball live at
   `dist/vue-lib/src/index.d.ts` instead. This breaks `vue-tsc` on every single Vue scaffold
   (frontend and fullstack — 8 of the 31 e2e scenarios). **This is a bug in the
   `@m2s2/vue-lib` package itself (the `m2s2-design-system` repo, not this one)** — there's a
   local checkout at `/Users/mgmaster24/projects/m2s2-design-system`. Not fixable from
   `m2s2-cli`'s templates except via an ugly local `.d.ts` shim workaround; the user hasn't yet
   said whether they want that or want it fixed upstream instead.
3. Coverage still not exercised in the e2e suite: `--auth yes` / `--billing yes` (only checked
   for file-existence in the fast/offline unit tests, never actually installed+built+linted), and
   CDK output isn't `cdk synth`-validated.
4. Nothing has been pushed to a remote. Commits made this session are local only.
