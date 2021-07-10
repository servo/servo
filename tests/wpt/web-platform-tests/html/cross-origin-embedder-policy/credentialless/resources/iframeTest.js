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

    send(parent_token, `
      let iframe = document.createElement("iframe");
      iframe.src = "${child_url}";
      document.body.appendChild(iframe);
    `);

    send(child_token, `
      send("${test_token}", "load");
    `);

    // There are no interoperable ways to check an iframe failed to load. So a
    // timeout is being used.
    // See https://github.com/whatwg/html/issues/125
    step_timeout(()=>send(test_token, "block"), 3000);

    assert_equals(await receive(test_token), expectation);
  }, description);
}

// A decorated version of iframeTest, adding CORP:cross-origin to the child.
const iframeTestCORP = function() {
  arguments[0] += ", CORP:cross-origin"; // description
  arguments[3] += corp_cross_origin;     // child_headers
  iframeTest(...arguments);
}
