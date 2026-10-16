# WPT Conformance Pass-Rate Baseline Report

**Generated:** 2026-10-16T14:50:22.419614+00:00 | **Source:** `meta_expectations`

## Expectations Baseline (Static tree analysis)

- **Total Expectation Files:** 18,753
- **Total Non-Default Entries:** 154,583

### Outcome Breakdown in `tests/wpt/meta`

| Status | Entry Count | Share of Failures |
|---|---:|---:|
| `FAIL` | 146,721 | 94.91% |
| `ERROR` | 2,591 | 1.68% |
| `NOTRUN` | 2,473 | 1.60% |
| `TIMEOUT` | 2,386 | 1.54% |
| `` | 226 | 0.15% |
| `PRECONDITION_FAILED` | 122 | 0.08% |
| `CRASH` | 31 | 0.02% |
| `[FAIL, PASS]` | 20 | 0.01% |
| `[PASS, FAIL]` | 5 | 0.00% |
| `[PASS, TIMEOUT]` | 2 | 0.00% |
| `[FAIL, TIMEOUT, PASS]` | 1 | 0.00% |
| `[TIMEOUT, OK]` | 1 | 0.00% |
| `[FAIL, TIMEOUT]` | 1 | 0.00% |
| `[TIMEOUT, FAIL]` | 1 | 0.00% |
| `[FAIL, PASS, TIMEOUT]` | 1 | 0.00% |
| `PASS` | 1 | 0.00% |

### Top Test Suites with Non-Pass Entries

| Test Suite | Top Outcomes |
|---|---|
| `css` | FAIL: 52,342, TIMEOUT: 248, NOTRUN: 209, ERROR: 141, [FAIL, PASS]: 11 |
| `html` | FAIL: 27,255, TIMEOUT: 1,027, NOTRUN: 570, : 223, ERROR: 158 |
| `editing` | FAIL: 23,781, ERROR: 6, TIMEOUT: 5, NOTRUN: 5 |
| `svg` | FAIL: 8,537, ERROR: 247, TIMEOUT: 86, NOTRUN: 35 |
| `input-events` | FAIL: 2,974, TIMEOUT: 6, NOTRUN: 4 |
| `webnn` | FAIL: 2,906, ERROR: 3, NOTRUN: 3 |
| `service-workers` | FAIL: 2,083, NOTRUN: 161, TIMEOUT: 72, ERROR: 21, CRASH: 1 |
| `WebCryptoAPI` | FAIL: 2,036, ERROR: 2 |
| `mimesniff` | FAIL: 1,928 |
| `fetch` | FAIL: 1,415, NOTRUN: 272, ERROR: 138, TIMEOUT: 82, [FAIL, PASS]: 9 |
| `url` | FAIL: 1,820, CRASH: 2, ERROR: 1 |
| `domparsing` | FAIL: 1,609 |
| `IndexedDB` | FAIL: 1,270, ERROR: 187, TIMEOUT: 17 |
| `webgl` | FAIL: 1,325 |
| `dom` | FAIL: 1,239, ERROR: 27, TIMEOUT: 25, NOTRUN: 7, PRECONDITION_FAILED: 6 |
| `wasm` | NOTRUN: 423, ERROR: 395, FAIL: 377, TIMEOUT: 102 |
| `content-security-policy` | FAIL: 767, TIMEOUT: 104, NOTRUN: 62, ERROR: 19, CRASH: 1 |
| `resource-timing` | FAIL: 814, NOTRUN: 90, TIMEOUT: 31, ERROR: 11, CRASH: 1 |
| `pointerevents` | FAIL: 750, PRECONDITION_FAILED: 78, TIMEOUT: 39, NOTRUN: 27, ERROR: 18 |
| `webaudio` | FAIL: 832, NOTRUN: 32, ERROR: 17, TIMEOUT: 15 |

## Cookie Partitioning (CHIPS) Status Note

Partitioned (`Partitioned`) cookie parsing and storage keying are implemented (`components/net/cookie.rs`), but `SameSiteContext.partition_key` is not currently populated with the top-level site on send. Partitioned cookies are currently stored but never sent.
