// One document embeds another in an iframe. Both are loaded from the network.
// Check whether or not the child can load.

// There are no interoperable ways to check an iframe failed to load. So a
// timeout is being used. See https://github.com/whatwg/html/issues/125
// Moreover, we want to track progress, managing timeout explicitly allows to
// get a per-test results, even in case of failure of one.
setup({ explicit_timeout: true });

const EXPECT_LOAD = "load";
const EXPECT_BLOCK = "block";

// Load a credentialless iframe. Control both the parent and the child headers.
// Check whether it loaded or not.
const embeddingTest = (description, {
  parent_headers,
  child_headers,
  child_origin,
  expectation,
}) => {
  // Default values:
  child_origin ||= globalThis.origin;
  parent_headers ||= "";
  child_headers||= "";

  const parent_origin = window.origin;

  promise_test_parallel(async test => {
    const parent_token = token();
    const parent_url = parent_origin + executor_path + parent_headers +
      `&uuid=${parent_token}`;

    const child_token = token();
    const child_url = child_origin + executor_path + child_headers +
      `&uuid=${child_token}`;

    // Create the parent:
    window.open(parent_url);
    add_completion_callback(() => send(parent_token, "close()"));

    // The parent creates its child:
    await send(parent_token, `
      const iframe = document.createElement("iframe");
      iframe.credentialless = true;
      iframe.src = "${child_url}";
      document.body.appendChild(iframe);
    `);

    // Ping the child to know whether it was allowed to load or not:
    const reply_token = token();
    await send(child_token, `
      send("${reply_token}", "load");
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
    step_timeout(() => send(reply_token, "block"), expectation == EXPECT_BLOCK
      ? 1500
      : 3500
    );

    assert_equals(await receive(reply_token), expectation);
  }, description);
};
