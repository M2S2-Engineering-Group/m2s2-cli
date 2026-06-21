use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn cmd() -> Command {
    Command::cargo_bin("m2s2").unwrap()
}

fn write_config(dir: &assert_fs::TempDir, framework: &str) {
    dir.child(".m2s2.json")
        .write_str(&format!("{{\n  \"framework\": \"{framework}\"\n}}\n"))
        .unwrap();
}

// ── Component — React ─────────────────────────────────────────────────────────

#[test]
fn generate_react_component() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "component", "MyCard", "--framework", "react"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/components/MyCard/MyCard.tsx")
        .assert(predicate::path::exists());
    tmp.child("src/components/MyCard/MyCard.scss")
        .assert(predicate::path::exists());
    tmp.child("src/components/MyCard/index.ts")
        .assert(predicate::path::exists());
}

#[test]
fn generate_react_component_kebab_name() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "component", "my-card", "--framework", "react"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/components/MyCard/MyCard.tsx")
        .assert(predicate::path::exists());
}

#[test]
fn generate_react_component_snake_name() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "component", "my_card", "--framework", "react"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/components/MyCard/MyCard.tsx")
        .assert(predicate::path::exists());
}

// ── Component — Vue ───────────────────────────────────────────────────────────

#[test]
fn generate_vue_component() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "component", "MyCard", "--framework", "vue"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/components/MyCard/MyCard.vue")
        .assert(predicate::path::exists());
    tmp.child("src/components/MyCard/MyCard.scss")
        .assert(predicate::path::exists());
    tmp.child("src/components/MyCard/index.ts")
        .assert(predicate::path::exists());
}

// ── Component — Angular ───────────────────────────────────────────────────────

#[test]
fn generate_angular_component() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "component", "MyCard", "--framework", "angular"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/app/components/my-card/my-card.component.ts")
        .assert(predicate::path::exists());
    tmp.child("src/app/components/my-card/my-card.component.html")
        .assert(predicate::path::exists());
    tmp.child("src/app/components/my-card/my-card.component.scss")
        .assert(predicate::path::exists());
}

// ── Component — detects framework from config ─────────────────────────────────

#[test]
fn generate_component_detects_framework_from_config() {
    let tmp = assert_fs::TempDir::new().unwrap();
    write_config(&tmp, "react");
    cmd()
        .args(["generate", "component", "UserAvatar"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/components/UserAvatar/UserAvatar.tsx")
        .assert(predicate::path::exists());
}

// ── Component — duplicate fails ───────────────────────────────────────────────

#[test]
fn generate_component_duplicate_fails() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "component", "MyCard", "--framework", "react"])
        .current_dir(&tmp)
        .assert()
        .success();

    cmd()
        .args(["generate", "component", "MyCard", "--framework", "react"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// ── Page — React ──────────────────────────────────────────────────────────────

#[test]
fn generate_react_page() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "page", "Dashboard", "--framework", "react"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/pages/Dashboard/DashboardPage.tsx")
        .assert(predicate::path::exists());
    tmp.child("src/pages/Dashboard/DashboardPage.scss")
        .assert(predicate::path::exists());
    tmp.child("src/pages/Dashboard/index.ts")
        .assert(predicate::path::exists());
}

// ── Page — Vue ────────────────────────────────────────────────────────────────

#[test]
fn generate_vue_page() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "page", "Dashboard", "--framework", "vue"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/pages/Dashboard/DashboardPage.vue")
        .assert(predicate::path::exists());
    tmp.child("src/pages/Dashboard/DashboardPage.scss")
        .assert(predicate::path::exists());
    tmp.child("src/pages/Dashboard/index.ts")
        .assert(predicate::path::exists());
}

// ── Page — Angular ────────────────────────────────────────────────────────────

#[test]
fn generate_angular_page() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args(["generate", "page", "Dashboard", "--framework", "angular"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/app/pages/dashboard/dashboard.component.ts")
        .assert(predicate::path::exists());
    tmp.child("src/app/pages/dashboard/dashboard.component.html")
        .assert(predicate::path::exists());
    tmp.child("src/app/pages/dashboard/dashboard.component.scss")
        .assert(predicate::path::exists());
}

// ── Service — Angular ─────────────────────────────────────────────────────────

#[test]
fn generate_angular_service() {
    let tmp = assert_fs::TempDir::new().unwrap();
    write_config(&tmp, "angular");
    cmd()
        .args(["generate", "service", "Auth"])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("src/app/services/auth.service.ts")
        .assert(predicate::path::exists());
}

// ── Service — React (should fail) ─────────────────────────────────────────────

#[test]
fn generate_service_on_react_fails() {
    let tmp = assert_fs::TempDir::new().unwrap();
    write_config(&tmp, "react");
    cmd()
        .args(["generate", "service", "Auth"])
        .current_dir(&tmp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Angular"));
}
