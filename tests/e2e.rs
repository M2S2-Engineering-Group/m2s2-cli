//! End-to-end integration tests.
//!
//! These tests scaffold a real project, resolve live package versions, run the install step
//! (npm install / go mod tidy / python venv + pip), then exercise the project the way a
//! developer would: `m2s2 build`, `m2s2 lint`, `m2s2 test`, `m2s2 generate ...` followed by a
//! rebuild, and finally a smoke test of `m2s2 dev` (wait for the dev server's port to accept
//! connections, then kill it). They hit the network and take several minutes each.
//!
//! Run the full matrix with:
//!
//!   cargo test --test e2e -- --ignored --nocapture --test-threads=1
//!
//! `--test-threads=1` matters here: several scenarios start a dev server on a stack's fixed
//! default port (5173 for Vite, 4200 for `ng serve`, 8080 for Go/Node backends, 8000 for
//! uvicorn, 5000 for `flask run`), and two scenarios on the same stack running concurrently
//! would collide.
//!
//! Or a single scenario:
//!
//!   cargo test --test e2e new_react_frontend_e2e -- --ignored --nocapture
//!
//! If your shell sandboxes loopback network access or process signals between sibling
//! processes (some CI/agent sandboxes do), the dev-smoke-test step will report the dev
//! server's port as never coming up even when it genuinely did. Set
//! `M2S2_E2E_SKIP_DEV_ASSERT=1` to downgrade that specific check to a warning instead of a
//! hard failure; leave it unset for a real assertion.

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

fn cmd() -> Command {
    let mut c = Command::cargo_bin("m2s2").unwrap();
    c.timeout(Duration::from_secs(600));
    c
}

/// Run `m2s2 <args>` inside `dir`, asserting success. Panics with the process's stdout/stderr
/// (via assert_cmd's failure message) if it doesn't succeed.
fn run_ok(dir: &Path, args: &[&str]) {
    cmd()
        .args(args)
        .current_dir(dir)
        .timeout(Duration::from_secs(300))
        .assert()
        .success();
}

// ── `m2s2 dev` smoke testing ────────────────────────────────────────────────────

/// Poll `127.0.0.1:port` until something accepts a TCP connection or `timeout` elapses.
fn wait_for_port(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    false
}

#[cfg(unix)]
fn spawn_dev(dir: &Path) -> Child {
    use std::os::unix::process::CommandExt;
    let mut command = std::process::Command::new(assert_cmd::cargo::cargo_bin("m2s2"));
    command
        .arg("dev")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0); // own process group, so we can kill the whole tree we spawn
    command.spawn().expect("failed to spawn `m2s2 dev`")
}

#[cfg(not(unix))]
fn spawn_dev(dir: &Path) -> Child {
    let mut command = std::process::Command::new(assert_cmd::cargo::cargo_bin("m2s2"));
    command
        .arg("dev")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().expect("failed to spawn `m2s2 dev`")
}

/// Kill `child` and (on Unix) everything it spawned, e.g. the `go run` binary or `npm`'s child
/// process. A plain `Child::kill()` only kills the direct child, which leaks dev-server
/// processes that `m2s2 dev` forks.
#[cfg(unix)]
fn kill_dev(mut child: Child) {
    let pgid = child.id() as i32;
    let _ = std::process::Command::new("kill")
        .args(["-TERM", &format!("-{pgid}")])
        .status();
    std::thread::sleep(Duration::from_millis(500));
    let _ = std::process::Command::new("kill")
        .args(["-KILL", &format!("-{pgid}")])
        .status();
    let _ = child.wait();
}

#[cfg(not(unix))]
fn kill_dev(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Drain a child's stdout/stderr in background threads so the process never blocks on a full
/// pipe buffer while we're busy polling ports.
fn drain_output(child: &mut Child) {
    for stream in [
        child
            .stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
        child
            .stderr
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
    ]
    .into_iter()
    .flatten()
    {
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                line.clear();
            }
        });
    }
}

/// Start `m2s2 dev` in `dir`, wait for every port in `ports` to come up, then kill it. Asserts
/// that all ports came up within `timeout`.
fn smoke_test_dev(dir: &Path, ports: &[u16], timeout: Duration) {
    let mut child = spawn_dev(dir);
    drain_output(&mut child);

    let mut up = Vec::with_capacity(ports.len());
    for &port in ports {
        up.push((port, wait_for_port(port, timeout)));
    }

    kill_dev(child);

    let down: Vec<u16> = up
        .into_iter()
        .filter(|(_, ok)| !ok)
        .map(|(port, _)| port)
        .collect();
    if down.is_empty() {
        return;
    }

    let message = format!(
        "`m2s2 dev` in {} never opened port(s) {down:?} within {timeout:?}",
        dir.display()
    );

    // Some sandboxed shells block loopback connections/signals between sibling processes,
    // which makes this assertion unreliable there even when the dev server is genuinely up.
    // Set M2S2_E2E_SKIP_DEV_ASSERT=1 to downgrade it to a warning in that case; unset (the
    // default) for a real, hard assertion.
    if std::env::var("M2S2_E2E_SKIP_DEV_ASSERT").is_ok() {
        eprintln!("warning: {message} (M2S2_E2E_SKIP_DEV_ASSERT set, not failing)");
    } else {
        panic!("{message}");
    }
}

// ── Shared post-scaffold verification ───────────────────────────────────────────

/// Run `build`, `lint`, `test`, then `generate component/page` (+ optional extra `generate`
/// invocations, e.g. Angular's `generate service`) followed by a rebuild, then smoke-test `dev`
/// on `dev_port`.
fn verify_frontend_lifecycle(dir: &Path, dev_port: u16, extra_generate: &[&[&str]]) {
    run_ok(dir, &["build"]);
    run_ok(dir, &["lint"]);
    run_ok(dir, &["test"]);

    run_ok(dir, &["generate", "component", "SmokeWidget"]);
    run_ok(dir, &["generate", "page", "SmokeScreen"]);
    for extra in extra_generate {
        run_ok(dir, extra);
    }
    run_ok(dir, &["build"]);

    smoke_test_dev(dir, &[dev_port], Duration::from_secs(90));
}

/// Run `build`, `lint`, `test`, then smoke-test `dev` on `dev_port`. Backend `generate` targets
/// (story/test scaffolding) don't exist yet — see PLAN.md Phase 2 — so there's no generate step.
fn verify_backend_lifecycle(dir: &Path, dev_port: u16) {
    run_ok(dir, &["build"]);
    run_ok(dir, &["lint"]);
    run_ok(dir, &["test"]);

    smoke_test_dev(dir, &[dev_port], Duration::from_secs(60));
}

/// Run `build`, `lint`, `test` (dispatches to both `apps/web` and `apps/api`), then a
/// `generate component` against the frontend half (`apps/web`, since `generate` operates
/// relative to CWD and has no awareness of the monorepo layout) followed by a rebuild, then
/// smoke-test `dev` on both the web and api ports.
fn verify_fullstack_lifecycle(dir: &Path, web_port: u16, api_port: u16) {
    run_ok(dir, &["build"]);
    run_ok(dir, &["lint"]);
    run_ok(dir, &["test"]);

    let web = dir.join("apps/web");
    run_ok(&web, &["generate", "component", "SmokeWidget"]);
    run_ok(dir, &["build"]);

    smoke_test_dev(dir, &[web_port, api_port], Duration::from_secs(90));
}

// ── Default dev-server ports per stack ──────────────────────────────────────────

const PORT_VITE: u16 = 5173; // react, vue
const PORT_ANGULAR: u16 = 4200; // ng serve
const PORT_GO: u16 = 8080; // gin, echo, fiber (PORT env unset)
const PORT_NODE: u16 = 8080; // express, fastify (PORT env unset)
const PORT_FASTAPI: u16 = 8000; // uvicorn default
const PORT_FLASK: u16 = 5000; // flask run default

// ── Frontend ──────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn new_react_frontend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "react",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-app");
    dir.child("package.json").assert(predicate::path::exists());
    dir.child(".m2s2.json").assert(predicate::path::exists());
    dir.child("node_modules").assert(predicate::path::is_dir());

    verify_frontend_lifecycle(dir.path(), PORT_VITE, &[]);
}

#[test]
#[ignore]
fn new_angular_frontend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "angular",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-app");
    dir.child("package.json").assert(predicate::path::exists());
    dir.child(".m2s2.json").assert(predicate::path::exists());
    dir.child("node_modules").assert(predicate::path::is_dir());

    verify_frontend_lifecycle(
        dir.path(),
        PORT_ANGULAR,
        &[&["generate", "service", "SmokeData"]],
    );
}

#[test]
#[ignore]
fn new_vue_frontend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "vue",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-app");
    dir.child("package.json").assert(predicate::path::exists());
    dir.child(".m2s2.json").assert(predicate::path::exists());
    dir.child("node_modules").assert(predicate::path::is_dir());

    verify_frontend_lifecycle(dir.path(), PORT_VITE, &[]);
}

// ── Backend — Go ─────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn new_go_gin_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "go",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("go.mod").assert(predicate::path::exists());
    dir.child("go.sum").assert(predicate::path::exists());
    dir.child(".m2s2.json").assert(predicate::path::exists());

    verify_backend_lifecycle(dir.path(), PORT_GO);
}

#[test]
#[ignore]
fn new_go_echo_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "go",
            "--api-framework",
            "echo",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("go.mod").assert(predicate::path::exists());
    dir.child("go.sum").assert(predicate::path::exists());

    verify_backend_lifecycle(dir.path(), PORT_GO);
}

#[test]
#[ignore]
fn new_go_fiber_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "go",
            "--api-framework",
            "fiber",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("go.mod").assert(predicate::path::exists());
    dir.child("go.sum").assert(predicate::path::exists());

    verify_backend_lifecycle(dir.path(), PORT_GO);
}

// ── Backend — Node ────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn new_node_express_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "node",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("package.json").assert(predicate::path::exists());
    dir.child(".m2s2.json").assert(predicate::path::exists());
    dir.child("node_modules").assert(predicate::path::is_dir());

    verify_backend_lifecycle(dir.path(), PORT_NODE);
}

#[test]
#[ignore]
fn new_node_fastify_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "node",
            "--api-framework",
            "fastify",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("package.json").assert(predicate::path::exists());
    dir.child("node_modules").assert(predicate::path::is_dir());

    verify_backend_lifecycle(dir.path(), PORT_NODE);
}

// ── Backend — Python ──────────────────────────────────────────────────────────

#[test]
#[ignore]
fn new_python_fastapi_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "python",
            "--api-framework",
            "fastapi",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("requirements.txt")
        .assert(predicate::path::exists());
    dir.child(".m2s2.json").assert(predicate::path::exists());
    dir.child(".venv").assert(predicate::path::is_dir());
    dir.child(".venv/bin/pip").assert(predicate::path::exists());

    verify_backend_lifecycle(dir.path(), PORT_FASTAPI);
}

#[test]
#[ignore]
fn new_python_flask_backend_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--runtime",
            "python",
            "--api-framework",
            "flask",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-api");
    dir.child("requirements.txt")
        .assert(predicate::path::exists());
    dir.child(".venv").assert(predicate::path::is_dir());
    dir.child(".venv/bin/pip").assert(predicate::path::exists());

    verify_backend_lifecycle(dir.path(), PORT_FLASK);
}

// ── Fullstack ─────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn new_react_gin_fullstack_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "react",
            "--runtime",
            "go",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-app");
    dir.child(".m2s2.json").assert(predicate::path::exists());
    dir.child("apps/web/package.json")
        .assert(predicate::path::exists());
    dir.child("apps/web/node_modules")
        .assert(predicate::path::is_dir());
    dir.child("apps/api/go.mod")
        .assert(predicate::path::exists());
    dir.child("apps/api/go.sum")
        .assert(predicate::path::exists());

    verify_fullstack_lifecycle(dir.path(), PORT_VITE, PORT_GO);
}

#[test]
#[ignore]
fn new_vue_fastapi_fullstack_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "vue",
            "--runtime",
            "python",
            "--api-framework",
            "fastapi",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-app");
    dir.child("apps/web/package.json")
        .assert(predicate::path::exists());
    dir.child("apps/web/node_modules")
        .assert(predicate::path::is_dir());
    dir.child("apps/api/requirements.txt")
        .assert(predicate::path::exists());
    dir.child("apps/api/.venv")
        .assert(predicate::path::is_dir());

    verify_fullstack_lifecycle(dir.path(), PORT_VITE, PORT_FASTAPI);
}

#[test]
#[ignore]
fn new_angular_express_fullstack_e2e() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "angular",
            "--runtime",
            "node",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "no",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    let dir = tmp.child("my-app");
    dir.child("apps/web/package.json")
        .assert(predicate::path::exists());
    dir.child("apps/web/node_modules")
        .assert(predicate::path::is_dir());
    dir.child("apps/api/package.json")
        .assert(predicate::path::exists());
    dir.child("apps/api/node_modules")
        .assert(predicate::path::is_dir());

    verify_fullstack_lifecycle(dir.path(), PORT_ANGULAR, PORT_NODE);
}

// ── Fullstack — remaining frontend × backend combinations ──────────────────────
//
// The three scenarios above exercise one frontend/backend pair each; these fill in the rest of
// the 3×7 matrix. Frontend+Node-backend combos matter most here — that pairing is what exposed
// the fullstack version-merge bug (both sides resolve a `typescript` version independently, and
// merging them into one shared map silently dropped one side's).

macro_rules! fullstack_e2e {
    ($fn_name:ident, $framework:literal, $runtime:literal, $api_framework:literal, $web_port:expr, $api_port:expr) => {
        #[test]
        #[ignore]
        fn $fn_name() {
            let tmp = assert_fs::TempDir::new().unwrap();
            cmd()
                .args([
                    "new",
                    "my-app",
                    "--project-type",
                    "fullstack",
                    "--framework",
                    $framework,
                    "--runtime",
                    $runtime,
                    "--api-framework",
                    $api_framework,
                    "--auth",
                    "no",
                    "--billing",
                    "no",
                ])
                .current_dir(&tmp)
                .assert()
                .success();

            let dir = tmp.child("my-app");
            dir.child(".m2s2.json").assert(predicate::path::exists());

            verify_fullstack_lifecycle(dir.path(), $web_port, $api_port);
        }
    };
}

// react + {echo, fiber, express, fastify, fastapi, flask}  (react+gin above)
fullstack_e2e!(
    new_react_echo_fullstack_e2e,
    "react",
    "go",
    "echo",
    PORT_VITE,
    PORT_GO
);
fullstack_e2e!(
    new_react_fiber_fullstack_e2e,
    "react",
    "go",
    "fiber",
    PORT_VITE,
    PORT_GO
);
fullstack_e2e!(
    new_react_express_fullstack_e2e,
    "react",
    "node",
    "express",
    PORT_VITE,
    PORT_NODE
);
fullstack_e2e!(
    new_react_fastify_fullstack_e2e,
    "react",
    "node",
    "fastify",
    PORT_VITE,
    PORT_NODE
);
fullstack_e2e!(
    new_react_fastapi_fullstack_e2e,
    "react",
    "python",
    "fastapi",
    PORT_VITE,
    PORT_FASTAPI
);
fullstack_e2e!(
    new_react_flask_fullstack_e2e,
    "react",
    "python",
    "flask",
    PORT_VITE,
    PORT_FLASK
);

// vue + {gin, echo, fiber, express, fastify, flask}  (vue+fastapi above)
fullstack_e2e!(
    new_vue_gin_fullstack_e2e,
    "vue",
    "go",
    "gin",
    PORT_VITE,
    PORT_GO
);
fullstack_e2e!(
    new_vue_echo_fullstack_e2e,
    "vue",
    "go",
    "echo",
    PORT_VITE,
    PORT_GO
);
fullstack_e2e!(
    new_vue_fiber_fullstack_e2e,
    "vue",
    "go",
    "fiber",
    PORT_VITE,
    PORT_GO
);
fullstack_e2e!(
    new_vue_express_fullstack_e2e,
    "vue",
    "node",
    "express",
    PORT_VITE,
    PORT_NODE
);
fullstack_e2e!(
    new_vue_fastify_fullstack_e2e,
    "vue",
    "node",
    "fastify",
    PORT_VITE,
    PORT_NODE
);
fullstack_e2e!(
    new_vue_flask_fullstack_e2e,
    "vue",
    "python",
    "flask",
    PORT_VITE,
    PORT_FLASK
);

// angular + {gin, echo, fiber, fastify, fastapi, flask}  (angular+express above)
fullstack_e2e!(
    new_angular_gin_fullstack_e2e,
    "angular",
    "go",
    "gin",
    PORT_ANGULAR,
    PORT_GO
);
fullstack_e2e!(
    new_angular_echo_fullstack_e2e,
    "angular",
    "go",
    "echo",
    PORT_ANGULAR,
    PORT_GO
);
fullstack_e2e!(
    new_angular_fiber_fullstack_e2e,
    "angular",
    "go",
    "fiber",
    PORT_ANGULAR,
    PORT_GO
);
fullstack_e2e!(
    new_angular_fastify_fullstack_e2e,
    "angular",
    "node",
    "fastify",
    PORT_ANGULAR,
    PORT_NODE
);
fullstack_e2e!(
    new_angular_fastapi_fullstack_e2e,
    "angular",
    "python",
    "fastapi",
    PORT_ANGULAR,
    PORT_FASTAPI
);
fullstack_e2e!(
    new_angular_flask_fullstack_e2e,
    "angular",
    "python",
    "flask",
    PORT_ANGULAR,
    PORT_FLASK
);
