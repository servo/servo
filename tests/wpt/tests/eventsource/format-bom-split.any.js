// META: title=EventSource: byte order mark split across chunks

async_test(t => {
  // A byte order mark split after its first byte, then "data:1\n\ndata:2\n\n".
  // If the byte order mark is not stripped, the first line's field name is
  // "\ufeffdata" and the first message is "2".
  const source = new EventSource("/xhr/resources/content-in-chunks.py?type=text/event-stream" +
      "&chunk=ef&chunk=bbbf646174613a310a0a646174613a320a0a");
  t.add_cleanup(() => source.close());
  source.onmessage = t.step_func_done(event => {
    assert_equals(event.data, "1");
  });
}, "EventSource: byte order mark split across chunks");
