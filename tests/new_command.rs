use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn cmd() -> Command {
    Command::cargo_bin("m2s2").unwrap()
}

// ── Frontend ─────────────────────────────────────────────────────────────────

#[test]
fn new_react_frontend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "react",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app").assert(predicate::path::is_dir());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/package.json")
        .assert(predicate::path::exists());
}

#[test]
fn new_angular_frontend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "angular",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app").assert(predicate::path::is_dir());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/package.json")
        .assert(predicate::path::exists());
}

#[test]
fn new_vue_frontend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "vue",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app").assert(predicate::path::is_dir());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/package.json")
        .assert(predicate::path::exists());
}

// ── Backend — Go ─────────────────────────────────────────────────────────────

#[test]
fn new_go_gin_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/go.mod").assert(predicate::path::exists());
}

#[test]
fn new_go_echo_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/go.mod").assert(predicate::path::exists());
}

#[test]
fn new_go_fiber_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/go.mod").assert(predicate::path::exists());
}

// ── Backend — Node ───────────────────────────────────────────────────────────

#[test]
fn new_node_express_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/package.json")
        .assert(predicate::path::exists());
}

#[test]
fn new_node_fastify_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/package.json")
        .assert(predicate::path::exists());
}

// ── Backend — Python ─────────────────────────────────────────────────────────

#[test]
fn new_python_fastapi_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/requirements.txt")
        .assert(predicate::path::exists());
}

#[test]
fn new_python_flask_backend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api").assert(predicate::path::is_dir());
    tmp.child("my-api/requirements.txt")
        .assert(predicate::path::exists());
}

// ── Fullstack ─────────────────────────────────────────────────────────────────

#[test]
fn new_react_gin_fullstack() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app").assert(predicate::path::is_dir());
    tmp.child("my-app/.m2s2.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/web/package.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/apps/api/go.mod")
        .assert(predicate::path::exists());
}

// ── Config file content ───────────────────────────────────────────────────────

#[test]
fn new_react_writes_m2s2_config() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "frontend",
            "--framework",
            "react",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.m2s2.json")
        .assert(predicate::str::contains("\"framework\""))
        .assert(predicate::str::contains("\"react\""));
}
