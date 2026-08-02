# API Verification and Publish Preflight Architecture

## Status

Proposed architecture for validating article requests, credentials, target capabilities, and
versioned API contracts before `m2s2 publish` performs any create or update operation.

This design complements:

- `docs/publishing-platform-evaluation.md`
- `docs/ai-social-content-architecture.md`

## Problem

The current publish loop validates some target-specific conditions only when each target's
`publish` method runs. If an earlier target succeeds and a later target fails a predictable check,
the overall operation leaves a partial cross-post. A retry can then create duplicates on targets
that do not yet support updates or idempotency.

API verification must happen before the first external write. It must answer:

1. Is the source article valid?
2. Can every requested target construct a valid request?
3. Does the request conform to the pinned upstream contract?
4. Are credentials present and apparently valid?
5. Is the configured destination reachable and correctly identified?
6. Does each target support the requested create or update operation?

Preflight reduces preventable failures. It cannot guarantee publication will succeed because
permissions, rate limits, network state, and remote data can change between verification and write.

## Goals

- Run all predictable checks before any publish side effect.
- Make preflight the default behavior of `m2s2 publish`.
- Offer a preflight-only command mode suitable for local use and CI.
- Keep ordinary contract tests offline and deterministic.
- Use only read-only remote operations during remote preflight.
- Produce a structured report that is useful to humans and automation.
- Prepare the exact request that will subsequently be published.
- Prevent secrets and article contents from leaking into diagnostic output.
- Support target-specific checks without putting target logic in command glue.

## Non-goals

- Guaranteeing that a subsequent write will succeed.
- Creating and deleting test posts during normal preflight.
- Downloading OpenAPI or GraphQL schemas on every publish.
- Treating a generic HTTP `OPTIONS` response as proof of publish permission.
- Automatically fixing credentials, permissions, or article content.
- Rolling back posts that were successfully published.

## Command behavior

### Normal publish

```bash
m2s2 publish posts/my-article.md
```

Execution order:

```text
Parse article
     |
     v
Load configuration and secrets
     |
     v
Local preflight for every target
     |
     v
Read-only remote preflight for every target
     |
     v
Print preflight summary
     |
     v
Publish prepared requests sequentially
     |
     v
Persist each outcome immediately
```

No target is published when any blocking preflight check fails.

### Preflight only

```bash
m2s2 publish posts/my-article.md --preflight-only
```

This runs local and read-only remote checks, prints the report, and exits without publishing.

Suggested exit codes:

| Exit code | Meaning |
|---|---|
| 0 | Every required check passed |
| 1 | Validation or capability check failed |
| 2 | Configuration or credential check failed |
| 3 | Remote verification was inconclusive because of network/service availability |

If the project prefers one conventional nonzero exit code, expose the category in JSON output
instead.

### Offline verification

```bash
m2s2 publish posts/my-article.md --preflight-only --offline
```

Offline mode runs parsing, configuration-shape, request construction, capability, and pinned
contract checks without network access. It cannot prove credentials or remote destination state.
Offline results must say `not-run`, never `passed`, for remote checks.

`--offline` is valid only with `--preflight-only`; an actual publish necessarily requires network
access.

### Machine-readable report

```bash
m2s2 publish posts/my-article.md --preflight-only --format json
```

Human-readable output remains the default. JSON output enables CI policies and future orchestration.

## Check phases

### Phase 1: article validation

These checks run once for the canonical article:

- File exists, is readable UTF-8, and contains YAML frontmatter.
- Required fields are present and non-empty.
- Slug is non-empty and valid for every selected target.
- Article body is non-empty.
- Target list is non-empty and contains no duplicates.
- Canonical URL is a valid absolute HTTPS URL when supplied.
- Date is valid and normalized.
- Tags are non-empty after normalization and satisfy target limits.
- Cover-image URL is HTTP(S), or its local file exists and has a supported type.
- Local image size is within the configured platform/request limit.
- No secret-like fields are present in frontmatter.

Warnings should not block publishing unless the user enables a future strict mode. Examples include
a missing canonical URL or tags that will be truncated for DEV.

### Phase 2: configuration validation

Validate every selected target before building any client:

- Required configuration section exists.
- Required identifiers and endpoints are non-empty.
- Endpoint URLs use HTTPS, except explicit localhost/test configuration.
- Secrets resolve through the configured environment/file mechanism.
- No token is printed in errors or verbose output.
- Target names and configuration sections agree.
- Timeouts and retry policy are valid.
- Requested operation is supported by the target.

If `--update` is selected with any target that cannot update, preflight fails before all network
writes. This fixes the current case where the platform could update before DEV or Hashnode rejects
the operation.

### Phase 3: request preparation

Each adapter converts the article and operation into a `PreparedRequest`. This phase performs all
target-specific transformations:

- Tag truncation or rejection.
- Cover-image resolution.
- Markdown/HTML conversion where required.
- Target request-body serialization.
- Required header construction, excluding loggable secret values.
- URL and query construction.
- Typed response expectation.

The prepared request is validated against the adapter's pinned contract and stored in memory. The
publish phase sends this exact prepared request; it must not repeat transformation or hook execution.

This is important for `platform.body_command`: run the hook once during preparation, validate its
JSON, and reuse that output for publication. Running the command again after preflight could produce
a different request or repeat an unintended side effect.

### Phase 4: read-only remote verification

Remote checks verify as much as the public API safely exposes without creating content:

- DNS/TLS connection succeeds within a short timeout.
- Authentication token is accepted by a documented read endpoint.
- Configured account/publication/destination exists.
- Returned identity matches optional configured expectations.
- API version is accepted.
- Obvious plan or access errors are surfaced.

A read endpoint generally cannot prove write permission. Reports must distinguish `credential
accepted` from `publish permission confirmed`. Only label publish permission as confirmed when the
API exposes a documented, read-only capability or scope endpoint that proves it.

## Result model

```rust
enum CheckStatus {
    Passed,
    Warning,
    Failed,
    NotRun,
    Inconclusive,
}

enum CheckPhase {
    Article,
    Configuration,
    Request,
    Contract,
    Remote,
}

struct PreflightCheck {
    code: &'static str,
    target: Option<TargetKind>,
    phase: CheckPhase,
    status: CheckStatus,
    message: String,
    remediation: Option<String>,
}

struct PreflightReport {
    article_slug: String,
    operation: PublishOperation,
    checks: Vec<PreflightCheck>,
    publish_allowed: bool,
}
```

Check codes are stable machine identifiers, for example:

```text
article.frontmatter.valid
article.cover_image.readable
devto.tags.limit
devto.contract.request
devto.credentials.accepted
hashnode.publication.exists
platform.body.valid_json
platform.remote.inconclusive
```

Human messages may evolve; CI should depend on codes and statuses rather than message text.

## Proposed Rust architecture

```text
src/publish/
├── article.rs
├── config.rs
├── preflight.rs
├── prepared.rs
├── report.rs
├── state.rs
├── target.rs
└── targets/
    ├── devto.rs
    ├── hashnode.rs
    └── platform.rs
```

### Core operations

```rust
enum PublishOperation {
    Create,
    Update,
}

struct PreparedTarget {
    kind: TargetKind,
    request: PreparedRequest,
}

impl Target {
    fn prepare(
        &self,
        article: &Article,
        operation: PublishOperation,
    ) -> Result<PreparedTarget>;

    async fn verify_remote(
        &self,
        prepared: &PreparedTarget,
    ) -> Vec<PreflightCheck>;

    async fn execute(
        &self,
        prepared: PreparedTarget,
    ) -> Result<PublishOutcome>;
}
```

The exact Rust representation can remain closed-enum dispatch, consistent with the existing
`Target` design. A trait object is not required.

### Orchestration

```rust
let article = parse_and_validate(...)?;
let targets = build_targets(...)?;

let prepared = targets
    .iter()
    .map(|target| target.prepare(&article, operation))
    .collect::<Result<Vec<_>>>()?;

let report = preflight_all(&targets, &prepared, mode).await;
report.print(format)?;

if !report.publish_allowed || args.preflight_only {
    return report.into_result();
}

execute_and_record(targets, prepared, state).await
```

All targets are prepared before remote verification. All remote verifications complete before the
first call to `execute`.

## Contract verification

Runtime preflight uses a contract version compiled into the binary; it does not download schemas.

### DEV/Forem

- Pin the official OpenAPI v1 document in `tests/contracts/forem/`.
- Validate request fixtures against the create/update schema during tests.
- Ensure every prepared request includes:

```text
Accept: application/vnd.forem.api-v1+json
api-key: <redacted>
```

- Use `GET https://dev.to/api/users/me` as the read-only credential/identity check.
- Treat HTTP 401 as credential failure.
- Persist the numeric article ID returned by create for later updates.

Official reference: https://developers.forem.com/api/v1

### Hashnode

- Pin the introspected GraphQL schema in `tests/contracts/hashnode/schema.graphql`.
- Validate the exact mutation document against the pinned schema during tests/build maintenance.
- Use a minimal read-only query for the configured publication during remote preflight.
- Confirm the returned publication ID matches configuration.
- Treat top-level GraphQL `errors`, missing `data`, null publication, and malformed JSON distinctly.
- Do not claim write permission is confirmed unless Hashnode exposes a read-only capability field
  that proves it.
- Persist the returned post ID for future updates.

Official reference: https://docs.hashnode.com/quickstart/introduction

### Generic platform

The generic platform adapter cannot assume a universal health, identity, or capability endpoint.
Add optional configuration:

```toml
[platform]
endpoint = "https://api.example.com"
path = "/admin/blog"
token_env = "M2S2_PLATFORM_TOKEN"
preflight_path = "/admin/me"
expected_identity = "author@example.com"
```

When `preflight_path` is configured, issue a documented GET and optionally validate the response
with configured expectations. When it is absent:

- Validate URL, request body, authentication configuration, and local image data.
- Mark remote identity/write capability `inconclusive`.
- Do not send `OPTIONS`, `HEAD`, an empty POST, or a synthetic draft unless the target explicitly
  documents that operation as safe.

If a platform needs custom remote verification, a future `preflight_command` hook may receive
redacted/configured context. It must be explicitly enabled and documented as trusted executable
code, like `body_command`.

## Future targets

### LinkedIn

- Validate required `Linkedin-Version` and `X-Restli-Protocol-Version` headers locally.
- Verify token identity through the documented OIDC user-info flow when the granted scopes permit.
- Verify configured organization role using a documented organization-access endpoint before
  organization posting.
- Do not infer `w_member_social` or `w_organization_social` merely from successful sign-in.
- Treat a sunset API version as a blocking preflight failure.

Posts API reference:
https://learn.microsoft.com/en-us/linkedin/marketing/community-management/shares/posts-api

### X

- Use `GET https://api.x.com/2/users/me` to verify the authenticated user.
- Confirm the returned user ID matches optional configuration.
- Validate every thread post before creating the first post.
- Record post IDs after every successful thread step.

Official user endpoint: https://docs.x.com/x-api/users/get-my-user

### CoderLegion and Medium

These remain manual-draft targets. Preflight validates only artifact generation and formatting;
there is no supported remote publishing capability to verify.

## Failure policy

### Blocking failures

- Invalid article or request.
- Missing target configuration.
- Unsupported operation.
- Contract mismatch.
- Invalid or rejected credentials.
- Configured destination not found.
- Definite plan/permission rejection.
- Local cover image unsupported by any requested target.

### Warnings

- Tag truncation explicitly accepted by policy.
- Missing canonical URL when not required.
- Remote API does not expose a read-only write-capability check.
- Generic platform has no configured preflight endpoint.

### Inconclusive remote failures

Timeouts, DNS failures, TLS failures, HTTP 429, and transient 5xx responses are not proof that
credentials or requests are invalid. The default publish command should still stop before writes
and ask the user to retry. A future override may be considered, but it should be explicit and noisy.

## Timeouts and retries

- Use a short connect timeout for remote preflight.
- Use a bounded overall request timeout.
- Retry only safe read operations for network errors, 429, and selected 5xx statuses.
- Honor `Retry-After`.
- Use bounded exponential backoff with jitter.
- Never retry authentication or validation failures.
- Preflight retries and publish retries must have separate policies.

## Security

- Redact API keys, bearer tokens, cookies, and authorization headers.
- Do not include secrets in `PreflightReport` or JSON output.
- Do not print full response bodies from authentication endpoints by default.
- Limit captured error bodies and redact common token patterns.
- Do not send article content to identity/health endpoints.
- Treat `body_command` and any future `preflight_command` as trusted local code.
- Prefer environment-variable credentials over repository-local plaintext secrets.
- Refuse redirects from HTTPS to HTTP for credentialed requests.

## Publication state

Preflight and idempotency are related but separate. After preflight passes and publishing begins,
persist each successful outcome immediately:

```json
{
  "schemaVersion": 1,
  "articleSlug": "my-article",
  "sourceHash": "sha256:...",
  "targets": {
    "devto": {
      "remoteId": "123456",
      "url": "https://dev.to/example/my-article",
      "status": "published"
    },
    "hashnode": {
      "remoteId": "post-id",
      "url": "https://example.hashnode.dev/my-article",
      "status": "published"
    }
  }
}
```

Preflight checks state before create:

- If the same source hash is already published, skip safely.
- If the slug exists with a different hash, require update or explicit direction.
- If a prior run partially succeeded, publish only missing targets.

This is what makes recovery safe after failures that preflight cannot prevent.

## Test strategy

### Unit tests

- Stable check codes and status aggregation.
- Warning versus blocking policy.
- Article and configuration validation.
- Target capability matrices.
- Request preparation.
- Secret redaction.
- State-based create/update/skip decisions.

### Contract tests

- DEV requests and response fixtures against pinned OpenAPI.
- Hashnode operations against the pinned GraphQL schema.
- Platform default-body JSON schema owned by M2S2.
- Contract version/checksum metadata is present.

### Mocked remote tests

- Valid and invalid credentials.
- Wrong publication/account identity.
- Rate limiting and `Retry-After`.
- Timeouts and transient errors.
- Malformed success responses.
- Remote verification never uses a write method.
- No publish request occurs when any preflight check fails.
- All prepared requests are created before the first external write.
- `body_command` executes once and its prepared output is reused.

### CLI integration tests

- `--preflight-only` performs no create/update request.
- `--offline` performs no network request.
- JSON report contains stable codes and no secrets.
- Unsupported multi-target update fails before all writes.
- A local image incompatible with one target prevents publishing to every target.
- A fully passing preflight proceeds to publish.
- Partial prior state resumes only missing targets.

## Delivery plan

### Phase 1: eliminate predictable partial publishes

- Add `PublishOperation`, `PreparedTarget`, and `PreflightReport`.
- Move target-specific validation into `prepare`.
- Prepare all targets before sending writes.
- Add `--preflight-only` and `--offline`.
- Add the DEV version header.
- Fail malformed success responses.

### Phase 2: read-only remote verification

- Add DEV authenticated-user verification.
- Add Hashnode publication verification.
- Add optional generic-platform preflight endpoint.
- Add bounded timeouts, safe-read retries, and JSON reporting.

### Phase 3: contract automation and recovery

- Pin the Forem OpenAPI and Hashnode GraphQL contracts.
- Add scheduled upstream contract-diff workflows.
- Persist remote IDs and publication state.
- Implement safe update/resume behavior.

## Acceptance criteria

The feature is complete when:

1. Every selected target is locally prepared before any write request.
2. A predictable failure on one target prevents writes to all targets.
3. Preflight-only mode sends no create, update, or delete request.
4. Offline mode performs no network access.
5. Remote preflight uses documented read-only endpoints only.
6. DEV requests validate against pinned OpenAPI and Hashnode operations against a pinned schema.
7. Reports distinguish passed, failed, not-run, warning, and inconclusive checks.
8. Secrets never appear in human or JSON output.
9. The exact prepared request validated by preflight is the request sent during publish.
10. Tests prove a failed preflight produces zero publishing side effects.

