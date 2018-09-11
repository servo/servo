test(() => {
  assert_throws(new TypeError(), () => navigator.sendBeacon("...", new ReadableStream()));
}, "sendBeacon() with a stream does not work due to the keepalive flag being set");
