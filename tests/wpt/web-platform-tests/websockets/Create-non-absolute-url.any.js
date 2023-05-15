[
  "test",
  "?",
  null,
  123,
].forEach(input => {
  test(() => {
    const url = new URL(input, location);
    url.protocol = "ws";
    const ws = new WebSocket(input);
    assert_equals(ws.url, url.href);
    ws.close();
  }, `Create WebSocket - Pass a non absolute URL: ${input}`);
});
