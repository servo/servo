// META: script=resources/wait-for-messages.js

function testNavigation(url) {
  return async (t) => {
    // Start waiting for messages before inserting the child frame, to avoid any
    // race conditions.
    const messagesPromise = waitForMessages(3);

    const iframe = document.createElement("iframe");
    iframe.src = url;
    document.body.appendChild(iframe);

    const messages = await messagesPromise;
    assert_array_equals(messages, ["initial", "inner", "destination"]);
  }
}

promise_test(
    testNavigation("resources/child-navigates-parent-location-initial.html"),
    "Child document navigates same-origin parent via document.location");

promise_test(
    testNavigation("resources/child-navigates-parent-submit-initial.html"),
    "Child document navigates same-origin parent via form submission");
