// META: title=EventSource: blob URL

async_test(t => {
  const blob = new Blob(["data: hello\n\n"], { type: "text/event-stream" });
  const url = URL.createObjectURL(blob);
  const source = new EventSource(url);
  // Revoke the blob URL immediately after creating the EventSource.
  // The claim taken by EventSource should keep the blob alive for the connection.
  URL.revokeObjectURL(url);
  source.onmessage = t.step_func_done(e => {
    assert_equals(e.data, "hello");
    source.close();
  });
  source.onerror = t.unreached_func("error should not fire");
}, "EventSource keeps blob URL alive after revokeObjectURL");
