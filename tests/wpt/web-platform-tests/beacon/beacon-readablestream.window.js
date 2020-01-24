test(() => {
  assert_throws_js(TypeError, () => navigator.sendBeacon("...", new ReadableStream()));
}, "sendBeacon() with a stream does not work due to the keepalive flag being set");
