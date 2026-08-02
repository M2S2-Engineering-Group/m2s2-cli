# M2S2 Platform Content Delivery Architecture

## Status

Proposed architecture for moving cross-platform content delivery into the Go-based M2S2 platform
while retaining the Rust CLI as a local authoring, validation, and control surface.

This document builds on:

- `docs/api-verification-preflight.md`
- `docs/ai-social-content-architecture.md`
- `docs/publishing-platform-evaluation.md`

## Product intent

The M2S2 platform will become a content delivery system: authors create one canonical article,
release an immutable version, validate it against every configured destination, deploy it to those
destinations, and observe the result.

The initial implementation serves M2S2's admin blog editor and publishing workflow. The same domain
model and APIs should support a future multi-tenant SaaS product without requiring the first release
to implement every SaaS capability.

## Architectural decision

The platform API is the product boundary. The CLI is a client of that API.

The platform, not the CLI, owns:

- Destination credentials and OAuth refresh tokens.
- Remote API verification and contract versions.
- Immutable releases and publishing runs.
- Asynchronous orchestration and target queues.
- Destination adapters and retries.
- Publication state, approvals, and audit history.
- AI-generated derivative drafts when generation runs as a hosted workflow.
- Analytics ingestion and billing/usage enforcement.

The CLI continues to own:

- Reading local Markdown and assets.
- Canonical article parsing and local validation.
- Local previews and deterministic rendering.
- Content hashing before upload.
- Authentication to the M2S2 platform.
- Creating releases through the platform API.
- Requesting preflight, deployment, approval, and retry operations.
- Displaying remote run status.

## Boundary summary

| Concern | CLI | Platform |
|---|---:|---:|
| Parse local Markdown | Owner | Revalidate on ingestion |
| Validate local files and images | Owner | Verify uploaded artifacts |
| Render canonical article | Owner | Store immutable snapshot |
| Compute source hash | Owner | Recompute and verify |
| Store destination credentials | Never | Owner |
| Verify remote credentials | Never | Owner |
| Perform remote preflight | Request/display only | Owner |
| Execute destination adapters | Legacy direct mode only | Owner |
| Queue, retry, and resume work | Never | Owner |
| Persist remote IDs and URLs | Cache/display only | Source of truth |
| Generate AI drafts locally | Optional authoring mode | Hosted workflow owner |
| Approve social drafts | Request/display only | Owner |
| Collect analytics | Display only | Owner |

The CLI stops at an authenticated platform API request. Once the platform accepts an immutable
release or publishing command, all external side effects are the platform's responsibility.

## Context diagram

```text
                  Local machine

  Markdown + assets
          |
          v
      m2s2 CLI
  parse / validate / hash
          |
          | HTTPS + platform identity
          v
  ------------------------------------------------ boundary
          |
          v
      Platform API (Go)
          |
          +--> article releases
          +--> publishing runs
          +--> credentials / policies
          +--> preflight orchestration
          +--> target work queues
          |
          v
      Go publishing workers
          |
          +--> M2S2 canonical site
          +--> DEV / Forem
          +--> Hashnode
          +--> LinkedIn
          +--> X
          +--> future destinations
```

The M2S2 admin editor enters the architecture inside the platform boundary. It uses the same Go
application services as the public platform API instead of calling the CLI.

## Core workflow

### Admin dashboard flow

1. The author edits an article in the M2S2 admin dashboard.
2. The author requests publication.
3. The Go API validates and stores the canonical article.
4. The platform resolves/uploads the public cover image.
5. The platform creates an immutable `ArticleRelease`.
6. The platform creates a `PublishingRun` and durable outbox event.
7. The API returns the release ID and run ID without waiting for external platforms.
8. A dispatcher publishes the outbox event.
9. A preflight worker prepares and verifies all selected external destinations.
10. If blocking preflight fails, no external destination receives a write.
11. If preflight passes, the orchestrator fans out one idempotent job per destination.
12. Each worker records its target outcome immediately.
13. The dashboard reads/streams the publishing-run state.
14. AI-generated LinkedIn/X content pauses in `awaiting_approval` until approved.

The canonical M2S2 article is published first. External syndication is asynchronous and may be
partially successful; publication state and idempotency make that condition recoverable.

### CLI flow

1. The CLI reads and locally validates Markdown and assets.
2. It computes a deterministic source hash.
3. It creates or reuses an immutable release through the platform API.
4. It requests remote preflight or deployment.
5. It prints the publishing-run ID.
6. With `--wait`, it polls or streams status until the run reaches a terminal/approval state.
7. Retry and approval commands call platform endpoints; they never use destination credentials.

## Go service structure

The exact folders should follow the existing platform repository conventions. The logical package
boundaries should remain recognizable:

```text
internal/publishing/
├── domain/
│   ├── article_release.go
│   ├── publishing_run.go
│   ├── target_delivery.go
│   ├── generated_draft.go
│   └── events.go
├── application/
│   ├── create_release.go
│   ├── start_run.go
│   ├── preflight.go
│   ├── execute_target.go
│   ├── retry_target.go
│   └── approve_draft.go
├── ports/
│   ├── release_repository.go
│   ├── run_repository.go
│   ├── target_repository.go
│   ├── queue.go
│   ├── secret_store.go
│   ├── snapshot_store.go
│   └── content_generator.go
├── adapters/
│   ├── devto/
│   ├── hashnode/
│   ├── linkedin/
│   ├── x/
│   └── generic_platform/
├── infrastructure/
│   ├── persistence/
│   ├── messaging/
│   ├── secrets/
│   └── observability/
└── transport/
    ├── http/
    └── events/
```

Domain and application packages must not import AWS SDK packages or destination HTTP clients.
Infrastructure implements ports and wires concrete services at the application boundary.

## Domain model

### Article

The mutable editorial record used by the admin editor. It is not itself deployed externally.

```text
Article
- ID
- WorkspaceID
- Slug
- CurrentDraft
- Revision
- EditorialStatus
- CreatedAt
- UpdatedAt
```

### ArticleRelease

An immutable deployment artifact.

```text
ArticleRelease
- ID
- WorkspaceID
- ArticleID
- Version
- SourceHash
- SchemaVersion
- Title
- Slug
- Summary
- Excerpt
- Tags
- Markdown
- CanonicalURL
- CoverImageURL
- SnapshotKey
- CreatedBy
- CreatedAt
```

A release must never fetch "the current article" during delivery. Workers use the immutable
snapshot referenced by the release.

### PublishingRun

One attempt to deploy an article release according to a target policy.

```text
PublishingRun
- ID
- WorkspaceID
- ReleaseID
- Operation          create | update | unpublish
- Status             queued | preflighting | blocked | deploying |
                     awaiting_approval | succeeded | partially_failed | failed
- IdempotencyKey
- RequestedTargets
- CreatedBy
- CreatedAt
- UpdatedAt
```

### TargetDelivery

The durable state for one release, operation, and destination.

```text
TargetDelivery
- ID
- RunID
- WorkspaceID
- Target
- Mode               automatic | approval | remote_draft | manual
- Status             queued | preflighting | ready | deploying |
                     awaiting_approval | succeeded | failed | skipped
- AttemptCount
- PreparedHash
- RemoteID
- RemoteURL
- LastErrorCode
- LastErrorMessage
- Retryable
- StartedAt
- CompletedAt
```

### GeneratedDraft

```text
GeneratedDraft
- ID
- RunID
- Target
- PromptVersion
- Provider
- Model
- SourceHash
- Content
- Status             generated | edited | approved | rejected | published
- ApprovedBy
- ApprovedAt
```

## State transitions

```text
PublishingRun

queued
  -> preflighting
      -> blocked
      -> deploying
          -> awaiting_approval
          -> succeeded
          -> partially_failed
          -> failed
```

```text
TargetDelivery

queued
  -> preflighting
      -> ready
          -> deploying
              -> succeeded
              -> failed
          -> awaiting_approval
              -> deploying
              -> rejected
      -> skipped
      -> failed
```

State changes use conditional writes/version checks so duplicate messages cannot move a completed
delivery backward.

## API surface

Version the platform contract independently from the CLI binary:

```text
POST /v1/releases
GET  /v1/releases/{releaseId}

POST /v1/releases/{releaseId}/preflight
POST /v1/releases/{releaseId}/deploy

GET  /v1/publishing-runs/{runId}
GET  /v1/publishing-runs/{runId}/targets

POST /v1/publishing-runs/{runId}/targets/{target}/retry
POST /v1/publishing-runs/{runId}/targets/{target}/approve
POST /v1/publishing-runs/{runId}/targets/{target}/reject
```

The API must publish an OpenAPI document used by the Rust platform client and API compatibility
tests.

### Create release

```http
POST /v1/releases
Authorization: Bearer <platform-token>
Idempotency-Key: <workspace>:<slug>:<source-hash>
Content-Type: application/json
```

```json
{
  "workspaceId": "ws_m2s2",
  "article": {
    "schemaVersion": 1,
    "title": "Architecture Is Built for Change",
    "slug": "architecture-is-built-for-change",
    "summary": "Why adaptability is an architectural quality.",
    "tags": ["architecture", "software-design"],
    "markdown": "# Architecture Is Built for Change\n\n..."
  },
  "sourceHash": "sha256:...",
  "assets": []
}
```

The server recomputes and verifies the hash. A repeated idempotency key returns the existing release.

### Deploy release

```json
{
  "targets": ["platform", "devto", "hashnode", "linkedin", "x"],
  "operation": "create",
  "waitForApproval": true
}
```

The response is asynchronous:

```json
{
  "releaseId": "rel_123",
  "runId": "run_456",
  "status": "queued"
}
```

## Events

Use versioned event types:

```text
content.release.created.v1
content.publish.requested.v1
content.preflight.completed.v1
content.target.requested.v1
content.target.completed.v1
content.draft.generated.v1
content.draft.approved.v1
```

Example:

```json
{
  "eventType": "content.publish.requested.v1",
  "eventId": "evt_789",
  "occurredAt": "2026-07-31T20:00:00Z",
  "workspaceId": "ws_m2s2",
  "releaseId": "rel_123",
  "runId": "run_456",
  "operation": "create",
  "targets": ["devto", "hashnode", "linkedin", "x"]
}
```

Events contain identifiers and immutable references, not article bodies, API keys, access tokens,
or OAuth refresh tokens.

## Reliability and delivery semantics

Use at-least-once messaging. Exactly-once delivery across external publishing APIs is not available.

### Idempotency

The logical target-operation key is:

```text
workspace-id + release-id + target + operation
```

Before executing a target job, the worker conditionally claims the corresponding `TargetDelivery`.
If it is already successful, the duplicate job exits successfully without another API call.

Persist a returned remote ID before acknowledging the queue message. Updates use the stored remote
ID rather than searching by title or assuming the canonical slug matches a destination slug.

### Outbox

Creating an article release/publishing run and requesting its asynchronous work must be durable.
Write an outbox record in the same database transaction or conditional batch as the domain state.
A dispatcher publishes undispatched records and marks them delivered only after the message bus
accepts them.

If the current datastore cannot atomically write both records, use a recoverable `dispatch_pending`
state and a reconciliation process that republishes undispatched runs.

### Retry policy

- Retry network failures, HTTP 429, and selected 5xx responses.
- Honor `Retry-After`.
- Use bounded exponential backoff with jitter.
- Do not retry authentication, validation, or definite permission failures automatically.
- Limit attempts per target and expose manual retry.
- A target retry never reruns successful sibling targets.

### No automatic rollback

Do not delete successful external posts because another destination failed. Record partial success
and retry the missing destination. Unpublish is an explicit operation with its own policy and audit
record.

## Preflight

Preflight follows `docs/api-verification-preflight.md` with one hosted-system adjustment: the
canonical M2S2 article may already be published before external preflight begins.

For external destinations:

1. Load the immutable release.
2. Load workspace target policy and credentials.
3. Prepare every target request.
4. Validate requests against pinned API contracts.
5. Perform documented read-only remote identity/capability checks.
6. Store a preflight report and prepared-request hash.
7. If any blocking check fails, dispatch no external target write.
8. If all blocking checks pass, create/fan out target jobs.

Prepared bodies may be encrypted or reconstructed deterministically at execution. If reconstructed,
the worker must verify that the resulting hash equals the preflight `PreparedHash` before sending.

## Hosted AI generation

AI generation consumes an immutable release and produces derivative drafts. It never edits the
release.

```text
ArticleRelease
    -> prompt + target policy
    -> provider request
    -> typed response validation
    -> GeneratedDraft
    -> human approval
    -> destination worker
```

Provider keys are platform secrets. Prompts and output schemas are versioned. Drafts retain source
hash, provider, model, and prompt version so the system can determine whether an article update
made an approved draft stale.

Initially:

- DEV and Hashnode: automatic publishing.
- LinkedIn and X: generated draft plus approval.
- CoderLegion: manual-submission artifact.

## AWS deployment recommendation

Use the platform's existing AWS conventions. A minimal serverless deployment can map the logical
components as follows:

| Logical component | AWS implementation |
|---|---|
| Go platform API | Existing API runtime/Lambda |
| Immutable snapshots/assets | S3 with versioning |
| Domain state/outbox | Existing database; DynamoDB transactions if applicable |
| Domain events | EventBridge |
| Target work queues | SQS standard queues with DLQs |
| Go workers | Lambda consumers |
| Credentials | Secrets Manager |
| Encryption | KMS |
| Logs/metrics/alarms | CloudWatch and existing OpenTelemetry pipeline |

Step Functions is not required for the MVP. The publishing-run state already provides workflow
visibility. Introduce Step Functions only if orchestration becomes difficult to understand or
approval/scheduling requirements materially benefit from its state-machine semantics.

## Security and tenant isolation

- Every domain record includes `WorkspaceID`.
- Repository methods require workspace scope; do not fetch by resource ID alone.
- Queue consumers verify the workspace relationship among event, release, run, target, and secret.
- Destination credentials live in Secrets Manager or equivalently encrypted secret storage.
- Do not include secrets in queues, logs, errors, OpenTelemetry attributes, or API responses.
- Platform access tokens use narrow scopes such as `release:create`, `release:deploy`, `run:read`,
  and `draft:approve`.
- CI/service tokens are workspace-scoped.
- OAuth refresh is centralized in the platform and protected by conditional updates/locking.
- External response bodies are bounded and redacted before persistence.
- Audit create, deploy, approve, retry, and unpublish operations with actor identity.

The initial M2S2-only implementation may use one workspace, but the workspace boundary must exist
in domain keys and repository interfaces from the start.

## Observability

Each request/event/job carries:

```text
trace-id
workspace-id
release-id
run-id
target-delivery-id
event-id
attempt
```

Metrics:

- Runs created, succeeded, partially failed, and failed.
- Preflight failures by stable check code.
- Delivery latency by target.
- Adapter success/error/rate-limit counts.
- Retry and DLQ counts.
- Approval wait time.
- AI generation duration and token usage when returned by providers.

Never use article bodies, prompts, generated content, tokens, or credentials as metric dimensions.

## Admin dashboard capability

The existing blog editor should add a publishing panel rather than block the editor's publish
request until all destinations finish.

```text
Release 4
Canonical M2S2       Published       View
DEV                  Published       View
Hashnode             Failed          Retry
LinkedIn             Awaiting review Review
X                    Awaiting review Review
CoderLegion          Draft ready     Copy
```

Required actions:

- View release and source hash.
- View per-target status and safe error summary.
- Retry one failed target.
- Review/edit/approve/reject a generated draft.
- View remote URLs.
- Start update propagation for a new article release.
- View actor and audit history.

## CLI integration

### Commands

```text
m2s2 auth login|status|logout
m2s2 workspace list|use

m2s2 content validate <file>
m2s2 content preview <file>
m2s2 content generate <file> --for <targets>
m2s2 content publish <file>
m2s2 content status [run-id] [--watch]
m2s2 content retry <run-id> --target <target>
m2s2 content approve <run-id> --target <target>
```

Keep `m2s2 publish <file>` as a compatibility alias during migration.

### Local project config

```toml
# .m2s2/config.toml — safe to commit

[platform]
profile = "production"
workspace = "m2s2-engineering"

[content]
articles_dir = "articles"
assets_dir = "assets"

[publish]
execution = "remote"
default_targets = ["platform", "devto", "hashnode"]
wait = true

[approvals]
linkedin = true
x = true
```

The global profile contains platform URLs. Platform tokens belong in the OS credential manager;
CI uses a workspace-scoped `M2S2_TOKEN`. Destination credentials never return to the CLI.

### Direct-mode migration

The current Rust adapters remain temporarily available:

```bash
m2s2 content publish article.md --execution direct
m2s2 content publish article.md --execution remote
```

Direct mode is for backward compatibility, adapter development, and possible self-hosting. Remote
mode is opt-in until the hosted platform is proven. A configured workspace may later make remote
the default, but the CLI must print the chosen execution mode before performing side effects.

## API/client compatibility

- The Go platform publishes versioned OpenAPI.
- Generate or contract-test the Rust API client from that specification.
- The CLI sends its version and supported article schema version.
- The platform returns a structured compatibility error when the CLI/schema is unsupported.
- Additive response fields do not break older clients.
- Breaking changes require a new API or article-schema version.
- Normal CLI tests use a mocked platform server and pinned OpenAPI; no real account is required.

## MVP scope

### In scope

- Single M2S2 workspace represented through the future tenant boundary.
- Existing admin editor as a source.
- Immutable releases and canonical M2S2 publication.
- Publishing runs and target-delivery state.
- Outbox, queues, retries, and DLQs.
- DEV and Hashnode automatic delivery.
- LinkedIn and X draft generation with approval.
- CoderLegion manual draft.
- CLI authentication, release upload, remote preflight, publish, status, and retry.
- Admin status/approval panel.

### Out of scope

- Public self-service signup and billing.
- Arbitrary customer CMS ingestion.
- Dozens of adapters.
- Fully autonomous social publishing.
- Cross-platform analytics beyond run state and remote URLs.
- Team/agency roles beyond the minimum workspace/actor model.
- Automatic rollback/deletion.

## Delivery phases

### Phase A: domain foundation in Go

- Define versioned article/release schemas.
- Implement `ArticleRelease`, `PublishingRun`, and `TargetDelivery`.
- Add repository ports, idempotency, and outbox.
- Publish initial OpenAPI.
- Add run-status endpoints.

### Phase B: M2S2 dashboard dogfood

- Connect admin publish to immutable release creation.
- Publish canonical content and cover image first.
- Add preflight orchestration.
- Implement DEV and Hashnode Go workers/adapters.
- Add per-target status and retry UI.

### Phase C: AI-assisted distribution

- Generate LinkedIn and X drafts.
- Add approval/edit/reject UI and endpoints.
- Produce CoderLegion manual artifact.
- Record prompt/model/source versions.

### Phase D: CLI remote client

- Add platform authentication and workspace selection.
- Add `.m2s2/config.toml`.
- Add release upload, remote preflight, publish, status, retry, and approval commands.
- Retain explicit direct mode.

### Phase E: SaaS hardening

- Enforce workspace isolation across all repositories and jobs.
- Add customer connection onboarding and encrypted OAuth/token management.
- Add usage metering, plans, audit retention, and support diagnostics.
- Add external source/webhook APIs and selected CMS adapters based on demand.

## Acceptance criteria

1. Admin publish returns after canonical release and durable workflow creation, without waiting for
   external destinations.
2. Every worker operates on an immutable article release.
3. Blocking external preflight failure produces zero external target writes.
4. Duplicate events/jobs do not create duplicate external posts.
5. A target failure does not rerun or delete successful sibling targets.
6. Destination credentials never reach the browser or CLI.
7. The dashboard and CLI display the same platform-owned publishing state.
8. LinkedIn/X drafts require approval by default.
9. CLI direct mode and remote mode are explicit during migration.
10. Go platform and Rust client contracts are verified against versioned OpenAPI.
11. Every state-changing operation is workspace-scoped and audited.
12. The system can resume a partially completed run using stored remote IDs and target state.

