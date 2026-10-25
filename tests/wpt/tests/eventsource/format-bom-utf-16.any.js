// META: title=EventSource: UTF-16 byte order mark

// Event streams are always decoded as UTF-8, so a UTF-16 byte order mark turns
// into two U+FFFD at the start of the first line. That line's field name is then
// "\ufffd\ufffddata", which is ignored, so the first message is "2".
for (const [name, bom] of [["UTF-16LE", "%FF%FE"], ["UTF-16BE", "%FE%FF"]]) {
  async_test(t => {
    const source = new EventSource("resources/message.py?message=" + bom +
        "data%3A1%0A%0Adata%3A2");
    t.add_cleanup(() => source.close());
    source.onmessage = t.step_func_done(event => {
      assert_equals(event.data, "2");
    });
    source.onerror = t.unreached_func("The stream ended without a message");
  }, "EventSource: " + name + " byte order mark is not used");
}
