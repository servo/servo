// META: script=/common/subset-tests-by-key.js
// META: variant=?include=integrity-none
// META: variant=?include=integrity-pass
// META: variant=?include=integrity-fail
// META: timeout=long

// Given `{ digest: "...", body: "...", cors: true }`:
function scriptURL(data) {
  data.type = "application/javascript";
  let params = new URLSearchParams(data);
  return "./resource.py?" + params.toString();
}

const executable_body = "window.hello = `world`;";
const unreached_body = "assert_unreached(`This code should not execute.`);";
const executable_hashes = {
  "sha-256": "PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=",
  "sha-384": "M5blqNh7AvXO/52MpQtxNMMV4B9uoKLMkdTte7k4mQz11WZDhH3P4QLWkvOA7llb",
  "sha-512": "6qaEeboWnnFooKiwqnorS3SbkLk5rZcqoSsgEeB97srB0WIH6hJk2QDevHAen7gym6/jW244Ogf5MhZMjPYFrA=="
};
const well_formed_but_incorrect_hashes = {
  "sha-256": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  "sha-384": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
  "sha-512": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
};

const EXPECT_BLOCKED = "block";
const EXPECT_LOADED = "loaded";
function generate_test(data, expectation, desc) {
  subsetTestByKey("integrity-none", async_test, t => {
    let s = document.createElement('script');
    s.src = scriptURL(data);
    if (expectation == EXPECT_BLOCKED) {
      s.onerror = t.step_func_done(e => {
        assert_equals("error", e.type);
      });
      s.onload = t.unreached_func("Script should not execute.");
    } else {
      s.onload = t.step_func_done(e => {
        assert_equals("load", e.type);
      });
      s.onerror = t.unreached_func("Script should not fail.");
    }
    document.body.appendChild(s);
  }, "No `integrity` attribute + " + desc);

  subsetTestByKey("integrity-pass", async_test, t => {
    let s = document.createElement('script');
    s.src = scriptURL(data);
    s.crossorigin = "anonymous";
    s.integrity = `sha256-${executable_hashes['sha-256']}`
    if (expectation == EXPECT_BLOCKED) {
      s.onerror = t.step_func_done(e => {
        assert_equals("error", e.type);
      });
      s.onload = t.unreached_func("Script should not execute.");
    } else {
      s.onload = t.step_func_done(e => {
        assert_equals("load", e.type);
      });
      s.onerror = t.unreached_func("Script should not fail.");
    }
    document.body.appendChild(s);
  }, "Matching `integrity` attribute + " + desc);

  subsetTestByKey("integrity-fail", async_test, t => {
    let s = document.createElement('script');
    s.src = scriptURL(data);
    s.crossorigin = "anonymous";
    s.integrity = `sha512-${well_formed_but_incorrect_hashes['sha-512']}`
    s.onerror = t.step_func_done(e => {
      assert_equals("error", e.type);
    });
    s.onload = t.unreached_func("Script should not execute.");
    document.body.appendChild(s);
  }, "Mismatching `integrity` attribute always blocks: " + desc);
}

// No header.
generate_test(
  { body: executable_body },
  EXPECT_LOADED,
  "No header: loads.");

let good_header_list = [];
let bad_header_list = [];
let mixed_header_list = [];
for (const key in executable_hashes) {
  let good_header = `${key}=:${executable_hashes[key]}:`;
  good_header_list.push(good_header);
  let bad_header = `${key}=:${well_formed_but_incorrect_hashes[key]}:`;
  bad_header_list.push(bad_header);
  mixed_header_list.push(good_header, bad_header);

  // - Good single headers:
  generate_test({
      body: executable_body,
      digest: good_header
    },
    EXPECT_LOADED,
    `Good ${key} header: loads.`);

  // - Good multiple headers:
  generate_test({
      body: executable_body,
      digest: `${good_header},${good_header}`
    },
    EXPECT_LOADED,
    `Repeated ${key} header: loads.`);

  generate_test({
      body: executable_body,
      digest: good_header_list.join(",")
    },
    EXPECT_LOADED,
    `Multiple good headers (previous += ${key}): loads.`);

  // - Bad single headers:
  generate_test({
      body: executable_body,
      digest: bad_header
    },
    EXPECT_BLOCKED,
    `Bad ${key} header: blocked.`);

  // - Bad multiple headers:
  generate_test({
      body: executable_body,
      digest: `${bad_header},${bad_header}`
    },
    EXPECT_BLOCKED,
    `Repeated ${key} header: blocked.`);

  generate_test({
      body: executable_body,
      digest: bad_header_list.join(",")
    },
    EXPECT_BLOCKED,
    `Multiple bad headers (previous += ${key}): blocked.`);
}

// - Mixed headers.
generate_test({
    body: executable_body,
    digest: mixed_header_list.join(","),
  },
  EXPECT_BLOCKED,
  `Mixed good and bad headers: blocked.`);
