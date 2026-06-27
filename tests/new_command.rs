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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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
            "--auth",
            "no",
            "--billing",
            "no",
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

// ── Auth & billing flags ──────────────────────────────────────────────────────

#[test]
fn new_with_auth_yes_writes_config() {
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
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.m2s2.json")
        .assert(predicate::str::contains("\"auth\": true"))
        .assert(predicate::str::contains("\"billing\": false"));
}

#[test]
fn new_with_billing_yes_writes_config() {
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
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.m2s2.json")
        .assert(predicate::str::contains("\"auth\": false"))
        .assert(predicate::str::contains("\"billing\": true"));
}

#[test]
fn new_with_auth_and_billing_yes_writes_config() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "angular",
            "--api-framework",
            "gin",
            "--auth",
            "yes",
            "--billing",
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.m2s2.json")
        .assert(predicate::str::contains("\"auth\": true"))
        .assert(predicate::str::contains("\"billing\": true"));
}

#[test]
fn new_defaults_auth_and_billing_to_false() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.m2s2.json")
        .assert(predicate::str::contains("\"auth\": false"))
        .assert(predicate::str::contains("\"billing\": false"));
}

// ── Auth template files ───────────────────────────────────────────────────────

#[test]
fn auth_yes_react_creates_auth_files() {
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
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/auth/amplify.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/auth/AuthProvider.tsx")
        .assert(predicate::path::exists());
}

#[test]
fn auth_no_react_omits_auth_files() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/auth/AuthProvider.tsx")
        .assert(predicate::path::missing());
}

#[test]
fn auth_yes_vue_creates_auth_files() {
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
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/auth/amplify.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/auth/useAuth.ts")
        .assert(predicate::path::exists());
}

#[test]
fn auth_yes_angular_creates_auth_service() {
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
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/app/auth/auth.service.ts")
        .assert(predicate::path::exists());
}

#[test]
fn auth_yes_express_creates_middleware() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "express",
            "--auth",
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/src/middleware/auth.ts")
        .assert(predicate::path::exists());
}

#[test]
fn auth_yes_fastify_creates_plugin() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fastify",
            "--auth",
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/src/plugins/auth.ts")
        .assert(predicate::path::exists());
}

#[test]
fn auth_yes_gin_creates_middleware() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/internal/middleware/auth.go")
        .assert(predicate::path::exists());
}

// ── CDK infra files ───────────────────────────────────────────────────────────

#[test]
fn cdk_generated_for_frontend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/cdk/cdk.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/cdk/bin/app.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/cdk/package.json")
        .assert(predicate::str::contains("@m2s2/cdk-constructs"));
}

#[test]
fn cdk_generated_for_backend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/cdk/cdk.json")
        .assert(predicate::path::exists());
    tmp.child("my-api/cdk/bin/app.ts")
        .assert(predicate::path::exists());
    tmp.child("my-api/cdk/package.json")
        .assert(predicate::str::contains("@m2s2/cdk-constructs"));
}

#[test]
fn cdk_generated_for_fullstack() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "react",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/cdk/cdk.json")
        .assert(predicate::path::exists());
    tmp.child("my-app/cdk/bin/app.ts")
        .assert(predicate::str::contains("@m2s2/cdk-constructs"))
        .assert(predicate::str::contains("apps/web/dist"))
        .assert(predicate::str::contains("apps/api"));
}

#[test]
fn cdk_auth_construct_generated_when_auth_yes() {
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
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/cdk/bin/app.ts")
        .assert(predicate::str::contains("auth: true"));
}

#[test]
fn cdk_auth_construct_omitted_when_auth_no() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/cdk/bin/app.ts")
        .assert(predicate::str::contains("auth: true").not());
}

#[test]
fn cdk_billing_construct_generated_when_billing_yes() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "backend",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/cdk/bin/app.ts")
        .assert(predicate::str::contains("billing: true"));
}

#[test]
fn cdk_app_stack_references_pascal_name() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-saas-app",
            "--project-type",
            "frontend",
            "--framework",
            "react",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-saas-app/cdk/bin/app.ts")
        .assert(predicate::str::contains("MySaasApp"));
}

// ── Billing template files ────────────────────────────────────────────────────

#[test]
fn billing_yes_react_creates_billing_files() {
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
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/billing/stripe.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/billing/CheckoutButton.tsx")
        .assert(predicate::path::exists());
}

#[test]
fn billing_no_react_omits_billing_files() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/billing/stripe.ts")
        .assert(predicate::path::missing());
}

#[test]
fn billing_yes_vue_creates_billing_files() {
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
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/billing/stripe.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/billing/useCheckout.ts")
        .assert(predicate::path::exists());
}

#[test]
fn billing_yes_angular_creates_billing_service() {
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
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/app/billing/billing.service.ts")
        .assert(predicate::path::exists());
}

#[test]
fn billing_yes_express_creates_webhook() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/src/webhooks/stripe.ts")
        .assert(predicate::path::exists());
}

#[test]
fn billing_yes_gin_creates_webhook() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/internal/webhooks/stripe.go")
        .assert(predicate::path::exists());
}

#[test]
fn billing_yes_fastapi_creates_webhook() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fastapi",
            "--auth",
            "no",
            "--billing",
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/billing/stripe_webhook.py")
        .assert(predicate::path::exists());
}

// ── GitHub Actions workflows ──────────────────────────────────────────────────

#[test]
fn github_actions_generated_for_frontend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.github/workflows/ci.yml")
        .assert(predicate::path::exists());
    tmp.child("my-app/.github/workflows/deploy.yml")
        .assert(predicate::path::exists());
}

#[test]
fn github_actions_generated_for_backend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/.github/workflows/ci.yml")
        .assert(predicate::path::exists());
    tmp.child("my-api/.github/workflows/deploy.yml")
        .assert(predicate::path::exists());
}

#[test]
fn github_actions_generated_for_fullstack() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "react",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.github/workflows/ci.yml")
        .assert(predicate::path::exists());
    tmp.child("my-app/.github/workflows/deploy.yml")
        .assert(predicate::path::exists());
}

#[test]
fn github_actions_ci_includes_go_setup_for_go_backend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/.github/workflows/ci.yml")
        .assert(predicate::str::contains("setup-go"))
        .assert(predicate::str::contains("go-version"));
}

#[test]
fn github_actions_ci_includes_python_setup_for_python_backend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fastapi",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/.github/workflows/ci.yml")
        .assert(predicate::str::contains("setup-python"))
        .assert(predicate::str::contains("python-version"));
}

#[test]
fn github_actions_deploy_uses_fullstack_paths_for_fullstack() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "vue",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/.github/workflows/deploy.yml")
        .assert(predicate::str::contains("apps/web"))
        .assert(predicate::str::contains("build-frontend"));
}

// ── SETUP.md ──────────────────────────────────────────────────────────────────

#[test]
fn setup_md_generated_for_frontend() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/SETUP.md")
        .assert(predicate::path::exists());
}

#[test]
fn setup_md_generated_for_backend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/SETUP.md")
        .assert(predicate::path::exists());
}

#[test]
fn setup_md_generated_for_fullstack() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "react",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/SETUP.md")
        .assert(predicate::path::exists());
}

#[test]
fn setup_md_includes_go_prereq_for_go_backend() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/SETUP.md")
        .assert(predicate::str::contains("Go"));
}

#[test]
fn setup_md_includes_auth_section_when_auth_yes() {
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
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/SETUP.md")
        .assert(predicate::str::contains("Auth Setup"))
        .assert(predicate::str::contains("Cognito"));
}

#[test]
fn setup_md_omits_auth_section_when_auth_no() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/SETUP.md")
        .assert(predicate::str::contains("Auth Setup").not());
}

#[test]
fn setup_md_includes_billing_section_when_billing_yes() {
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
            "yes",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/SETUP.md")
        .assert(predicate::str::contains("Billing Setup"))
        .assert(predicate::str::contains("Stripe"));
}

#[test]
fn setup_md_fullstack_includes_apps_paths() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-app",
            "--project-type",
            "fullstack",
            "--framework",
            "vue",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/SETUP.md")
        .assert(predicate::str::contains("apps/web"))
        .assert(predicate::str::contains("apps/api"));
}

// ── Backend parity — CORS + /api/v1 ──────────────────────────────────────────

#[test]
fn echo_has_cors_and_api_v1() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "echo",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/internal/handlers/info.go")
        .assert(predicate::path::exists());
    tmp.child("my-api/internal/routes/routes.go")
        .assert(predicate::str::contains("CORSWithConfig"))
        .assert(predicate::str::contains("/api/v1"));
}

#[test]
fn fiber_has_cors_and_api_v1() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fiber",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/internal/handlers/info.go")
        .assert(predicate::path::exists());
    tmp.child("my-api/internal/routes/routes.go")
        .assert(predicate::str::contains("cors.New"))
        .assert(predicate::str::contains("/api/v1"));
}

#[test]
fn express_has_cors_middleware_and_api_v1() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "express",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/src/middleware/cors.ts")
        .assert(predicate::path::exists());
    tmp.child("my-api/src/routes/info.ts")
        .assert(predicate::path::exists());
    tmp.child("my-api/src/index.ts")
        .assert(predicate::str::contains("/api/v1"));
}

#[test]
fn fastify_has_cors_hook_and_api_v1() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fastify",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/src/routes/info.ts")
        .assert(predicate::path::exists());
    tmp.child("my-api/src/index.ts")
        .assert(predicate::str::contains("Access-Control-Allow-Origin"))
        .assert(predicate::str::contains("/api/v1"));
}

#[test]
fn fastapi_has_cors_middleware_and_api_v1() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fastapi",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/routers/info.py")
        .assert(predicate::path::exists());
    tmp.child("my-api/main.py")
        .assert(predicate::str::contains("CORSMiddleware"))
        .assert(predicate::str::contains("/api/v1"));
}

#[test]
fn flask_has_cors_and_api_v1() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "flask",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/routes/info.py")
        .assert(predicate::path::exists());
    tmp.child("my-api/main.py")
        .assert(predicate::str::contains("Access-Control-Allow-Origin"))
        .assert(predicate::str::contains("/api/v1"));
}

// ── Angular reference kit ─────────────────────────────────────────────────────

#[test]
fn angular_scaffolds_routed_pages() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/app/pages/home/home.component.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/app/pages/home/home.component.html")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/app/pages/about/about.component.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/app/pages/contact/contact.component.ts")
        .assert(predicate::path::exists());
}

#[test]
fn angular_scaffolds_api_service_and_environment() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/app/services/api.service.ts")
        .assert(predicate::path::exists());
    tmp.child("my-app/src/environments/environment.ts")
        .assert(predicate::path::exists());
}

#[test]
fn angular_routes_include_lazy_loaded_pages() {
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
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-app/src/app/app.routes.ts")
        .assert(predicate::str::contains("loadComponent"))
        .assert(predicate::str::contains("HomeComponent"))
        .assert(predicate::str::contains("AboutComponent"));
}

// ── Gin reference kit ─────────────────────────────────────────────────────────

#[test]
fn gin_scaffolds_cors_middleware() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/internal/middleware/cors.go")
        .assert(predicate::path::exists());
}

#[test]
fn gin_scaffolds_api_v1_info_handler() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "gin",
            "--auth",
            "no",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/internal/handlers/info.go")
        .assert(predicate::path::exists());
    tmp.child("my-api/internal/routes/routes.go")
        .assert(predicate::str::contains("/api/v1"))
        .assert(predicate::str::contains("middleware.CORS"));
}

#[test]
fn auth_yes_fastapi_creates_cognito_module() {
    let tmp = assert_fs::TempDir::new().unwrap();
    cmd()
        .args([
            "new",
            "my-api",
            "--project-type",
            "backend",
            "--api-framework",
            "fastapi",
            "--auth",
            "yes",
            "--billing",
            "no",
            "--skip-install",
            "--offline",
        ])
        .current_dir(&tmp)
        .assert()
        .success();

    tmp.child("my-api/auth/cognito.py")
        .assert(predicate::path::exists());
}
