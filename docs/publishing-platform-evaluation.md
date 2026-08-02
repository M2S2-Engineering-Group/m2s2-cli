# Publishing Platform Evaluation and Contract Validation

## Status

Research and architectural recommendation as of 2026-07-31. Platform APIs, access tiers, and
commercial terms change frequently; each adapter must be treated as a versioned external contract.

## Executive recommendation

Prioritize platforms in this order:

1. Harden the existing DEV/Forem adapter against the official OpenAPI contract.
2. Harden the existing Hashnode adapter against a pinned GraphQL schema.
3. Generate LinkedIn and X drafts through `m2s2 content generate`.
4. Add explicit LinkedIn and X publishing only after OAuth and account access are proven.
5. Treat CoderLegion and Medium as manual-draft destinations, not API adapters.
6. Add Ghost or WordPress only when a real M2S2/client destination requires one.
7. Consider Mastodon or Bluesky as later social-distribution adapters, not blog replicas.

The product should optimize for traffic back to the canonical M2S2 article. More destinations are
not automatically better; every adapter adds authentication, contract, retry, duplicate-post, and
content-formatting costs.

## Support matrix

| Platform | Publishing API | Machine-readable contract | Access caveat | Recommendation |
|---|---|---|---|---|
| DEV/Forem | Yes | OpenAPI 3 | API key and required versioned `Accept` header | Support now |
| Hashnode | Yes | GraphQL schema/introspection | Publishing access may depend on the publication's plan | Support now |
| LinkedIn | Yes | Documented REST schema; no OpenAPI artifact identified | OAuth permissions and organization roles | Generate first; publish second |
| X | Yes | Documented REST API | Approved app, user OAuth, and current usage/billing terms | Generate first; publish second |
| CoderLegion | No documented public publishing endpoint found | None found | Editorial submission through its web UI | Manual draft only |
| Medium | Legacy API only for existing tokens | Legacy documentation | No new integration tokens or integrations | Drop API adapter |
| Ghost | Yes | Stable Admin API documentation | Destination owner must run/use Ghost | Optional generic CMS target |
| WordPress | Yes | REST discovery/schema plus reference docs | Site-specific authentication and plugins | Optional generic CMS target |
| beehiiv | Yes, beta | API reference | Create-post API is currently Enterprise-only | Defer |
| Blogger | Yes | Google discovery/API reference | Google OAuth and limited strategic audience fit | Low priority |
| Mastodon | Yes | Documented REST API | Instance URL and OAuth registration vary | Later social target |
| Bluesky | Yes | AT Protocol lexicons | Different record/protocol model | Later social target |

## DEV/Forem

### Contract

Forem documents both API v0 and v1 with OpenAPI 3 specifications. V1 is the recommended API and
requires:

```text
Accept: application/vnd.forem.api-v1+json
api-key: <user-api-key>
```

The create-article contract supports title, Markdown body, published state, description, up to four
tags, an absolute main-image URL, and canonical URL. Updating an article is supported by numeric
article ID.

Official sources:

- https://developers.forem.com/api/
- https://developers.forem.com/api/v1
- https://developers.forem.com/contributing-guide/api

### Validation approach

- Vendor a pinned copy of the upstream v1 OpenAPI document under `tests/contracts/forem/`.
- Record the upstream source URL, retrieval date, and SHA-256 checksum in a small metadata file.
- Validate serialized request fixtures against the request schema.
- Validate successful and error response fixtures against the documented response schemas.
- Keep `httpmock` tests for behavior, headers, error mapping, and returned IDs.
- Add a scheduled CI job that downloads the current specification and reports a diff. It should
  open an issue or fail a non-release compatibility job, not silently rewrite the pinned contract.
- Add the required v1 `Accept` header to every request.

The adapter should persist the returned article ID. That ID enables an actual update operation and
safe retries instead of creating duplicates.

## Hashnode

### Contract

Hashnode exposes a GraphQL endpoint at:

```text
https://gql.hashnode.com
```

The schema, rather than an OpenAPI file, is the machine-readable contract. The GraphQL playground
and official documentation expose the available queries, mutations, inputs, and response fields.

Official sources:

- https://docs.hashnode.com/quickstart/introduction
- https://gql.hashnode.com

### Validation approach

- Retrieve the schema through GraphQL introspection during an explicit maintenance task.
- Pin the schema under `tests/contracts/hashnode/schema.graphql` with retrieval metadata and a
  checksum.
- Validate/compile the exact `publishPost` and future `updatePost` operations against that schema.
- Deserialize responses into typed structures; never treat malformed JSON, missing `data`, or a
  null post as success.
- Continue checking GraphQL's top-level `errors` array even when HTTP status is 200.
- Add a scheduled schema-diff job separate from normal offline tests.
- Persist the returned Hashnode post ID and URL for updates and idempotent recovery.

Schema introspection is a maintenance/build concern, not something the CLI should perform every
time a user publishes.

## LinkedIn

### Feasibility

LinkedIn's current Posts API supports organic text, image, video, document, and article posts. A
create request uses:

```text
POST https://api.linkedin.com/rest/posts
Authorization: Bearer <token>
Linkedin-Version: YYYYMM
X-Restli-Protocol-Version: 2.0.0
```

Member posting uses `w_member_social`. Organization posting uses `w_organization_social` and the
authenticated member must have an eligible role on the organization. Article posts do not scrape
link previews automatically; title, description, and optional uploaded thumbnail must be supplied.
The created post ID is returned in the `x-restli-id` response header.

Official source:

- https://learn.microsoft.com/en-us/linkedin/marketing/community-management/shares/posts-api

### Recommendation

Phase one should only generate `linkedin.md`. This delivers most of the time savings without
making OAuth approval a prerequisite.

Before building direct publishing, run a short feasibility spike:

1. Register the LinkedIn application.
2. Confirm `w_member_social` for the intended personal account.
3. Confirm `w_organization_social` and the M2S2 organization-page role if company posting matters.
4. Publish and delete a text-only test post in a non-production test window.
5. Record token lifetime and refresh behavior.

LinkedIn's contract should be represented with hand-maintained typed Rust request/response models,
golden fixtures derived from official examples, and tests that enforce both version headers. Pin the
`Linkedin-Version` in configuration/code and review it on a scheduled cadence because LinkedIn
sunsets dated Marketing API versions.

## X

### Feasibility

X supports creating posts through:

```text
POST https://api.x.com/2/tweets
```

It requires an approved developer app and a user access token obtained through OAuth 1.0a or OAuth
2.0 PKCE. A thread is a sequence of create calls: the first creates a post and each following call
uses the preceding post ID as `reply.in_reply_to_tweet_id`.

Official sources:

- https://docs.x.com/x-api/posts/manage-tweets/quickstart
- https://docs.x.com/x-api/fundamentals/rate-limits

### Recommendation

Generate and review `x.md` first. If direct thread publishing is added later:

- Pre-validate every post's effective length before the first API call.
- Persist every returned post ID immediately.
- Stop on the first failure and report the partial thread precisely.
- Resume from publication state rather than recreating the entire thread.
- Never automatically delete already-published posts as rollback.
- Recheck current X access and pricing immediately before implementation.

## CoderLegion

No public article-publishing API or machine-readable contract was found. Its documented workflow is
to create an account, submit through the platform, and wait for editorial review. Its site mentions
API access as a premium concept, but does not publish an article API specification. A recent
community feedback article also describes the lack of a public API.

Official/public sources:

- https://coderlegion.com/publish-with-us
- https://coderlegion.com/about-us

Do not reverse-engineer browser endpoints or automate the editor. That would be brittle and may
conflict with the site's terms. Instead, generate a `coderlegion.md` draft suitable for manual
submission. Contact CoderLegion directly and request partner/API documentation before implementing
an adapter.

## Medium

Medium states that it no longer issues new integration tokens and does not allow new integrations.
Existing tokens may continue to work, but that is not a viable foundation for a public CLI.

Official source:

- https://help.medium.com/hc/en-us/articles/213480228-API-Importing

Drop Medium from the adapter roadmap. If Medium still provides useful distribution, generate a
manual draft or use Medium's webpage-import feature. Do not build against undocumented browser
requests.

## Other blog destinations

### Ghost

Ghost has a well-documented Admin API that creates and updates posts, supports integration-token
authentication, and can accept HTML as source content.

Official sources:

- https://docs.ghost.org/admin-api
- https://docs.ghost.org/admin-api/posts/creating-a-post

Ghost is an excellent adapter technically, but it does not automatically provide a new audience;
it is most valuable if M2S2 or a client already uses Ghost. Implement it as a CMS destination when a
real deployment appears, not merely to increase the adapter count.

### WordPress

WordPress exposes `POST /wp/v2/posts` and supports drafts, scheduled posts, updates, categories,
tags, excerpts, and featured-media IDs.

Official source:

- https://developer.wordpress.org/rest-api/reference/posts/

WordPress has broad applicability for client work. Its complexity is site-specific authentication,
Markdown-to-HTML conversion, media upload, and plugin behavior. It is a good second generic CMS
adapter after the current configurable `platform` target.

### beehiiv

beehiiv now documents a create-post API, but it is beta and currently restricted to Enterprise
customers. It is therefore unsuitable as a general default adapter today.

Official source:

- https://developers.beehiiv.com/api-reference/posts/create

### Blogger

Google's Blogger v3 API supports inserting posts. It is contractually usable, but OAuth complexity
and audience fit make it lower priority for M2S2 than LinkedIn, DEV, Hashnode, or a newsletter.

Official source:

- https://developers.google.com/blogger/docs/3.0/reference/posts/insert

## Additional social distribution

Mastodon has a mature create-status API and supports an `Idempotency-Key`, making safe retries
better defined than on many social platforms. Bluesky's AT Protocol also has public schemas and a
create-record flow. Both are technically attractive later targets, but audience fit should be
validated through manually posted drafts before investing in adapters.

Mastodon source:

- https://docs.joinmastodon.org/methods/statuses/

## Adapter contract architecture

Use three layers of validation:

```text
Upstream machine-readable contract
              |
              v
     Pinned contract snapshot
              |
              v
 Compile/schema fixture validation
              |
              v
 Mocked behavioral contract tests
              |
              v
 Optional credentialed smoke test
```

### Offline release tests

Normal unit and integration tests must remain offline and deterministic. They validate against
pinned specs/schemas and local HTTP mocks. No developer credentials are required.

### Scheduled compatibility tests

A separate scheduled workflow retrieves current upstream contracts and compares them with the
pinned versions. Changes require human review. The workflow must not automatically accept a changed
contract or modify production request models.

### Credentialed smoke tests

Real publishing tests are manual or protected workflows. They should create drafts where the API
supports drafts, use dedicated test accounts/publications, and record remote IDs for cleanup. They
must never run on an ordinary pull request.

### Common adapter requirements

Every publishing adapter should provide:

- `preflight(article, operation)` with no external side effects.
- Typed create/update request and response models.
- Explicit connect and request timeouts.
- Redacted authentication errors.
- Rate-limit recognition and bounded retries.
- A stable remote post ID and URL in `PublishOutcome`.
- Publication-state persistence before the next target begins.
- Clear create, update, and unsupported-operation capabilities.
- Contract version metadata exposed in verbose diagnostics.

## Product decision

The near-term M2S2 distribution set should be:

```text
Canonical M2S2 article
├── M2S2 platform       publish automatically
├── DEV                 publish automatically
├── Hashnode            publish automatically
├── LinkedIn            AI-assisted draft; direct API after access spike
├── X                   AI-assisted thread; direct API after access/cost spike
└── CoderLegion         formatted manual-submission draft
```

Medium should not block the roadmap. Ghost and WordPress should remain demand-driven CMS adapters.
Measure referral traffic from the initial set before adding more destinations.

