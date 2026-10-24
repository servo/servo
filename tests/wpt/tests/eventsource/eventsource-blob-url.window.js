

async_test(t => {
  const blob = new Blob(["data: hello\n\n"], { type: "text/event-stream" });
  const url = URL.createObjectURL(blob);
  const source = new EventSource(url);
  URL.revokeObjectURL(url);
  source.onmessage = t.step_func_done(e => {
    assert_equals(e.data, "hello");
    source.close();
  });
  source.onerror = t.unreached_func("error should not fire");
}, "EventSource can connect to a blob URL");
