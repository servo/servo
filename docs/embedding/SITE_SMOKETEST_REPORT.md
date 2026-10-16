# Real-World Website Conformance Smoke Report

| Site | Category | Status | Time | Root Cause |
|---|---|---|---:|---|
| [Example Domain](https://example.com) | Basic HTML5 / Static Document | `PLANNED` | 0.0s | Awaiting binary build on runner |
| [Hacker News](https://news.ycombinator.com) | Table Layout / Lightweight Dynamic | `PLANNED` | 0.0s | Awaiting binary build on runner |
| [Wikipedia Main Page](https://en.wikipedia.org/wiki/Main_Page) | Complex Content / Media Queries / SVG | `PLANNED` | 0.0s | Awaiting binary build on runner |
| [DuckDuckGo Lite](https://lite.duckduckgo.com) | Forms / Query Parameters | `PLANNED` | 0.0s | Awaiting binary build on runner |
| [Httpbin JSON](https://httpbin.org/get) | Network / Headers / HTTPS | `PLANNED` | 0.0s | Awaiting binary build on runner |

## Root Cause Triage & Findings

### Example Domain (PLANNED)
- **URL:** `https://example.com`
- **Category:** Basic HTML5 / Static Document
- **Root Cause:** Awaiting binary build on runner

### Hacker News (PLANNED)
- **URL:** `https://news.ycombinator.com`
- **Category:** Table Layout / Lightweight Dynamic
- **Root Cause:** Awaiting binary build on runner

### Wikipedia Main Page (PLANNED)
- **URL:** `https://en.wikipedia.org/wiki/Main_Page`
- **Category:** Complex Content / Media Queries / SVG
- **Root Cause:** Awaiting binary build on runner

### DuckDuckGo Lite (PLANNED)
- **URL:** `https://lite.duckduckgo.com`
- **Category:** Forms / Query Parameters
- **Root Cause:** Awaiting binary build on runner

### Httpbin JSON (PLANNED)
- **URL:** `https://httpbin.org/get`
- **Category:** Network / Headers / HTTPS
- **Root Cause:** Awaiting binary build on runner

## Execution Instructions

Run this suite using `etc/ci/site_smoketest.py` against a built `servoshell` binary:

```bash
# Execute smoke test across target sites (requires compiled servoshell):
python etc/ci/site_smoketest.py --binary target/release/servoshell --output-markdown docs/embedding/SITE_SMOKETEST_REPORT.md --output-json docs/embedding/site_smoketest.json

# Generate placeholder planned status without running:
python etc/ci/site_smoketest.py --dry-run --output-markdown docs/embedding/SITE_SMOKETEST_REPORT.md
```
