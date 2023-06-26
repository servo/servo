// One document embeds another in an iframe. Both are loaded from the network.
// Depending on the response headers:
// - Cross-Origin-Embedder-Policy (COEP)
// - Cross-Origin-Resource-Policy (CORP)
// The child must load or must be blocked.
//
// What to do for:
// - COEP:credentialless
// - COEP:credentialless-on-children
// is currently an active open question. This test will be updated/completed
// later.

// There are no interoperable ways to check an iframe failed to load. So a
// timeout is being used. See https://github.com/whatwg/html/issues/125
// Moreover, we want to track progress, managing timeout explicitly allows to
// get a per-test results, even in case of failure of one.
setup({ explicit_timeout: true });

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

// Open a new window loaded with the given |headers|. The new document will
// execute any script sent toward the token it returns.
const newWindow = (headers) => {
  const executor_token = token();
  const url = same_origin + executor_path + headers + `&uuid=${executor_token}`;
  const w = window.open(url);
  add_completion_callback(() => w.close());
  return executor_token;
};

const EXPECT_LOAD = "load";
const EXPECT_BLOCK = "block";

// Load in iframe. Control both the parent and the child headers. Check whether
// it loads or not.
const iframeTest = function(
  description,
  parent_token,
  child_origin,
  child_headers,
  expectation
) {
  promise_test_parallel(async test => {
    const test_token = token();

    const child_token = token();
    const child_url = child_origin + executor_path + child_headers +
      `&uuid=${child_token}`;

    await send(parent_token, `
      let iframe = document.createElement("iframe");
      iframe.src = "${child_url}";
      document.body.appendChild(iframe);
    `);

    await send(child_token, `
      send("${test_token}", "load");
    `);

    // There are no interoperable ways to check an iframe failed to load. So a
    // timeout is being used.
    // See https://github.com/whatwg/html/issues/125
    // Use a shorter timeout when it is expected to be reached.
    // - The long delay reduces the false-positive rate. False-positive causes
    //   stability problems on bot, so a big delay is used to vanish them.
    //   https://crbug.com/1215956.
    // - The short delay avoids delaying too much the test(s) for nothing and
    //   timing out. False-negative are not a problem, they just need not to
    //   overwhelm the true-negative, which is trivial to get.
    step_timeout(()=>send(test_token, "block"), expectation == EXPECT_BLOCK
      ? 2000
      : 6000
    );

    assert_equals(await receive(test_token), expectation);
  }, description);
}

// A decorated version of iframeTest, adding CORP:cross-origin to the child.
const iframeTestCORP = function() {
  arguments[0] += ", CORP:cross-origin"; // description
  arguments[3] += corp_cross_origin;     // child_headers
  iframeTest(...arguments);
}
