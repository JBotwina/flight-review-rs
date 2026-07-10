# Railway template configuration

Railway templates are composed in the Railway dashboard; Railway does not
currently import a repository-owned template JSON file. This document is the
canonical template recipe for Flight Review. Railway supports Bucket resources
inside templates, so the finished template contains the application, a private
S3-compatible Bucket, and a small SQLite volume. The service-level build and
deploy settings are committed separately in `railway.json`.

## Template metadata

- **Name:** PX4 Flight Review with AI Analysis
- **Description:** Upload PX4 ULogs to Railway Object Storage, inspect
  browser-native flight plots, and generate evidence-backed flight debriefs
  with any compatible OpenRouter model.
- **Category:** Developer Tools
- **Source:** `https://github.com/JBotwina/flight-review-rs`
- **Branch:** `main`
- **Documentation:** `README.md#railway`

## Bucket: FlightReviewData

Add a **Bucket** resource to the template before configuring the application:

- **Resource name:** `FlightReviewData` (keep this exact name because the
  application variables below reference it)
- **Region:** The same region as the Flight Review service
- **Access:** Private (Railway's default and only supported bucket access mode)

The bucket stores uploaded ULogs, Parquet files, extracted metadata, and saved
AI analyses under the `flight-review/` prefix. Those objects are served through
the Rust API; bucket credentials and private object URLs never reach the
browser.

## Service: Flight Review

- **Source type:** GitHub repository
- **Root directory:** `/`
- **Config file:** `/railway.json`
- **Builder:** Dockerfile
- **Dockerfile:** `/Dockerfile`
- **Public networking:** Enabled; generate a Railway domain
- **Healthcheck:** `/health`
- **Replicas:** 1 (a Railway volume cannot be shared by multiple replicas)

### Volume

Attach one persistent volume to the service:

- **Name:** `flight-review-data`
- **Mount path:** `/data`

The volume contains only the SQLite database. Durable log artifacts live in the
`FlightReviewData` Bucket. Enable volume backups for production deployments.

### Variables

| Variable | Required | Default | Notes |
|----------|----------|---------|-------|
| `OPENROUTER_API_KEY` | Yes | — | Mark as a secret. Never bake it into the image. |
| `OPENROUTER_DEFAULT_MODEL` | No | `openrouter/auto` | Used for API uploads that do not select a model. |
| `OPENROUTER_APP_NAME` | No | `PX4 Flight Review` | OpenRouter attribution title. |
| `OPENROUTER_BASE_URL` | No | `https://openrouter.ai/api/v1` | Override only for a compatible gateway. |
| `RUST_LOG` | No | `info` | Rust tracing filter. |
| `SERVER_FEATURES` | Yes | `s3` | Docker build argument. SQLite is a default crate feature; `s3` adds Railway Bucket support. |
| `PUBLIC_MAPBOX_TOKEN` | No | — | Build-time browser token for maps. Redeploy after changing it. |
| `MAPBOX_ACCESS_TOKEN` | No | — | Runtime server token for reverse geocoding. |
| `STORAGE_URL` | Yes | `s3://${{FlightReviewData.BUCKET}}/flight-review` | Selects the Bucket and isolates app objects under one prefix. |
| `S3_ACCESS_KEY_ID` | Yes | `${{FlightReviewData.ACCESS_KEY_ID}}` | Bucket credential reference; mark as a sealed variable. |
| `S3_SECRET_ACCESS_KEY` | Yes | `${{FlightReviewData.SECRET_ACCESS_KEY}}` | Bucket credential reference; mark as a sealed variable. |
| `S3_REGION` | Yes | `${{FlightReviewData.REGION}}` | Railway currently returns `auto`. |
| `S3_ENDPOINT` | Yes | `${{FlightReviewData.ENDPOINT}}` | Railway's S3-compatible base endpoint. |
| `S3_URL_STYLE` | Yes | `virtual` | Railway Buckets use virtual-hosted-style URLs. |
| `SEED_LOGS` | Yes | `true` | On every start, idempotently checks and imports the five pinned public examples that are missing. |
| `SEED_LOGS_PUBLIC` | No | `true` | Makes seed examples visible in the default Browse view. |
| `SEED_AI_MODEL` | No | — | Optional model override; empty uses `OPENROUTER_DEFAULT_MODEL`. |

`PORT`, `RAILWAY_PUBLIC_DOMAIN`, and the volume variables are provided by
Railway. The server reads `PORT` automatically, derives OpenRouter's site URL
from `RAILWAY_PUBLIC_DOMAIN`, and converts the Bucket's base endpoint into the
virtual-hosted endpoint expected by its Rust S3 client.

The image contains only `seed/logs.json`, not the 268 MB of ULogs. Its startup
seed pass downloads exact pinned CDN objects, verifies their SHA-256 hashes,
and sends missing logs through the normal upload API. Existing filenames are
skipped, so restarts and redeployments do not create duplicates. A failed seed
pass does not take down the app and is retried on the next start. The
entrypoint initializes ownership on Railway's root-mounted volume and
immediately drops to the unprivileged `flightreview` user before starting the
server or seed process.

## Create and share the template

1. Deploy this repository to a new Railway project.
2. Add a Bucket named `FlightReviewData`, use Railway's variable-reference UI
   to add the six storage variables above, and attach the `/data` volume.
3. Generate a public domain and verify `/health`, one upload, and its AI debrief.
   Allow the background seed pass to finish, then confirm Browse contains five
   examples and the Bucket contains objects below `flight-review/<log UUID>/`.
4. Open **Project Settings → Generate Template from Project**.
5. Confirm the app, Bucket, domain, variables, and volume in the Template
   Composer, then create the template. Railway supports Buckets in templates,
   so deployments will receive an isolated Bucket instance and credentials.
6. Copy the generated template URL. Add Railway's deploy button to the README
   only after that URL exists, so the button cannot point to a stale template.
