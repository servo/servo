// META: script=/common/get-host-info.sub.js
// META: script=resources/wait-for-messages.js

function testNavigationFails(params) {
  return async (t) => {
    // Start waiting for messages before inserting the child frame, to avoid any
    // race conditions. Note that this would be racy if we executed tests
    // concurrently, thankfully `promise_test` executes sequentially. See also:
    // https://github.com/web-platform-tests/rfcs/pull/75
    const messagesPromise = waitForMessages(1);

    // Execute the test in an iframe, so that the document executing the test
    // is not navigated away mid-test in case of failure.
    const child = document.createElement("iframe");
    document.body.appendChild(child);
    t.add_cleanup(() => { document.body.removeChild(child); });

    const url = new URL(
        "resources/child-navigates-parent-cross-origin-inner.html",
        window.location);

    // Load the grandchild iframe from a different origin.
    url.host = get_host_info().REMOTE_HOST;

    for (const key in params || {}) {
      url.searchParams.set(key, params[key]);
    }

    const grandchild = child.contentDocument.createElement("iframe");
    grandchild.src = url;
    child.contentDocument.body.appendChild(grandchild);

    const messages = await messagesPromise;
    assert_array_equals(messages, ["error: SecurityError"]);
  }
}

promise_test(
    testNavigationFails(),
    "Child document attempts to navigate cross-origin parent via location");

promise_test(
    testNavigationFails({ "property": "hash" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.hash");

promise_test(
    testNavigationFails({ "property": "host" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.host");

promise_test(
    testNavigationFails({ "property": "hostname" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.hostname");

promise_test(
    testNavigationFails({ "property": "href" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.href");

promise_test(
    testNavigationFails({ "property": "pathname" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.pathname");

promise_test(
    testNavigationFails({ "property": "protocol" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.protocol");

promise_test(
    testNavigationFails({ "property": "reload" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.reload()");

promise_test(
    testNavigationFails({ "property": "replace" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.replace()");

promise_test(
    testNavigationFails({ "property": "search" }),
    "Child document attempts to navigate cross-origin parent via "+
    "location.search");

promise_test(
    testNavigationFails({ "property": "xxxNonExistent" }),
    "Child document attempts to navigate cross-origin parent via non-standard "+
    "location property");
