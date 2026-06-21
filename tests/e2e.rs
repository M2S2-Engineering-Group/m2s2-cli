//! End-to-end integration tests.
//!
//! These tests scaffold a real project, resolve live package versions, and run
//! the install step (npm install / go mod tidy / python venv + pip). They hit
//! the network and take several minutes each.
//!
//! Run manually with:
//!
//!   cargo test --test e2e -- --ignored --nocapture
//!
//! Or a single scenario:
//!
//!   cargo test --test e2e new_react_frontend_e2e -- --ignored --nocapture

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::time::Duration;

fn cmd() -> Command {
    let mut c = Command::cargo_bin("m2s2").unwrap();
    c.timeout(Duration::from_secs(600));
    c
}

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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/node_modules")
        .assert(predicate::path::is_dir());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/node_modules")
        .assert(predicate::path::is_dir());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/node_modules")
        .assert(predicate::path::is_dir());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/go.mod").assert(predicate::path::exists());
    tmp.child("my-api/go.sum").assert(predicate::path::exists());
    tmp.child("my-api/.m2s2.json")
        .assert(predicate::path::exists());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/go.mod").assert(predicate::path::exists());
    tmp.child("my-api/go.sum").assert(predicate::path::exists());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/go.mod").assert(predicate::path::exists());
    tmp.child("my-api/go.sum").assert(predicate::path::exists());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/node_modules")
        .assert(predicate::path::is_dir());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/node_modules")
        .assert(predicate::path::is_dir());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/requirements.txt")
        .assert(predicate::path::exists());
    tmp.child("my-api/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/.venv").assert(predicate::path::is_dir());
    tmp.child("my-api/.venv/bin/pip")
        .assert(predicate::path::exists());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/requirements.txt")
        .assert(predicate::path::exists());
    tmp.child("my-api/.venv").assert(predicate::path::is_dir());
    tmp.child("my-api/.venv/bin/pip")
        .assert(predicate::path::exists());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/web/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/web/node_modules")
        .assert(predicate::path::is_dir());
    tmp.child("my-app/apps/api/go.mod")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/api/go.sum")
        .assert(predicate::path::exists());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/apps/web/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/web/node_modules")
        .assert(predicate::path::is_dir());
    tmp.child("my-app/apps/api/requirements.txt")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/api/.venv")
        .assert(predicate::path::is_dir());
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
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/apps/web/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/web/node_modules")
        .assert(predicate::path::is_dir());
    tmp.child("my-app/apps/api/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/api/node_modules")
        .assert(predicate::path::is_dir());
}
