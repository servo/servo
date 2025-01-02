// META: global=window,dedicatedworker,sharedworker
// META: script=/common/subset-tests-by-key.js
// META: variant=?include=integrity-none
// META: variant=?include=integrity-pass
// META: variant=?include=integrity-fail
// META: timeout=long

// Given `{ digest: "...", body: "...", cors: true, type: "..." }`:
function resourceURL(data) {
  let params = new URLSearchParams(data);
  return "./resource.py?" + params.toString();
}

const hello_world = "hello world";
const hello_hashes = {
  "sha-256": "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=",
  "sha-384": "/b2OdaZ/KfcBpOBAOF4uI5hjA+oQI5IRr5B/y7g1eLPkF8txzmRu/QgZ3YwIjeG9",
  "sha-512": "MJ7MSJwS1utMxA9QyQLytNDtd+5RGnx6m808qG1M2G+YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw=="
};

const dlrow_olleh = hello_world.split("").reverse().join("");
const dlrow_hashes = {
  "sha-256": "vT+a3uWsoxRxVJEINKfH4XZpLqsneOzhFVY98Y3iIz0=",
  "sha-384": "rueKXz5kdtdmTpc6NbS9fCqr7z8h2mjNs43K9WUglTsZPJzKSUpR87dLs/FNemRN",
  "sha-512": "N/peuevAy3l8KpS0bB6VTS8vc0fdAvjBJKYjVo2xb6sB6LpDfY6YlrXkWeeXGrP07UXDXEu1K3+SaUqMNjEkxQ=="
};

const EXPECT_BLOCKED = "block";
const EXPECT_LOADED = "loaded";
function generate_test(data, expectation, desc) {
  subsetTestByKey("integrity-none", promise_test, test => {
    let fetcher = fetch(resourceURL(data));
    if (expectation == EXPECT_BLOCKED) {
      return promise_rejects_js(test, TypeError, fetcher);
    } else {
      return fetcher.then(async r => {
        assert_equals(r.status, 200, "Response status is 200.");
        assert_equals(await r.text(), data.body);
      });
    }
  }, "No integrity metadata + " + desc);

  subsetTestByKey("integrity-pass", promise_test, test => {
    // Force CORS for the integrity check below:
    const data_with_cors = structuredClone(data);
    data_with_cors.cors = true;

    // The integrity check should pass, and nothing about the
    // `Identity-Digest` check should be affected.
    let fetcher = fetch(resourceURL(data_with_cors), { integrity: `sha256-${hello_hashes['sha-256']}`, mode: "cors" });
    if (expectation == EXPECT_BLOCKED) {
      return promise_rejects_js(test, TypeError, fetcher);
    } else {
      return fetcher.then(async r => {
        assert_equals(r.status, 200, "Response status is 200.");
        assert_equals(await r.text(), data.body);
      });
    }
  }, "Good integrity metadata + " + desc);


  subsetTestByKey("integrity-fail", promise_test, test => {
    // Force CORS for the integrity check below:
    const data_with_cors = structuredClone(data);
    data_with_cors.cors = true;

    // The integrity check should fail, so the resource should be blocked,
    // even with matching `Identity-Digest`s.
    let fetcher = fetch(resourceURL(data_with_cors), { integrity: `sha256-${dlrow_hashes['sha-256']}`, mode: "cors" });
    return promise_rejects_js(test, TypeError, fetcher);
  }, "Bad integrity metadata blocks everything, even: " + desc);
}

// No header.
generate_test(
  { body: hello_world },
  EXPECT_LOADED,
  "No header: loads.");

let good_header_list = [];
let bad_header_list = [];
let mixed_header_list = [];
for (const key in hello_hashes) {
  let good_header = `${key}=:${hello_hashes[key]}:`;
  good_header_list.push(good_header);
  let bad_header = `${key}=:${dlrow_hashes[key]}:`;
  bad_header_list.push(bad_header);
  mixed_header_list.push(good_header, bad_header);

  // - Good single headers:
  generate_test({
      body: hello_world,
      digest: good_header
    },
    EXPECT_LOADED,
    `Good ${key} header: loads.`);

  // - Good multiple headers:
  generate_test({
      body: hello_world,
      digest: `${good_header},${good_header}`
    },
    EXPECT_LOADED,
    `Repeated ${key} header: loads.`);

  generate_test({
      body: hello_world,
      digest: good_header_list.join(",")
    },
    EXPECT_LOADED,
    `Multiple good headers (previous += ${key}): loads.`);

  // - Bad single headers:
  generate_test({
      body: hello_world,
      digest: bad_header
    },
    EXPECT_BLOCKED,
    `Bad ${key} header: blocked.`);

  // - Bad multiple headers:
  generate_test({
      body: hello_world,
      digest: `${bad_header},${bad_header}`
    },
    EXPECT_BLOCKED,
    `Repeated ${key} header: blocked.`);

  generate_test({
      body: hello_world,
      digest: bad_header_list.join(",")
    },
    EXPECT_BLOCKED,
    `Multiple bad headers (previous += ${key}): blocked.`);
}

// - Mixed headers.
generate_test({
    body: hello_world,
    digest: mixed_header_list.join(","),
  },
  EXPECT_BLOCKED,
  `Mixed good and bad headers: blocked.`);
