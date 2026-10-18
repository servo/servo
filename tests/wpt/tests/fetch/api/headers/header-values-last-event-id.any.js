// META: title=Fetch: non-ASCII Last-Event-ID header value
// META: global=window,worker

promise_test(async () => {
  const value = "caf\xC3\xA9";
  const response = await fetch("../resources/echo-header-bytes.py?headers=last-event-id|x-test", {
    headers: { "Last-Event-ID": value, "X-Test": value }
  });
  assert_equals(await response.text(), "last-event-id: 636166c3a9\nx-test: 636166c3a9");
}, "fetch() sends the bytes of a non-ASCII Last-Event-ID header value unchanged");
