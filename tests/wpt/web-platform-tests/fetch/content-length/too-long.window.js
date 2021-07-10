promise_test(async t => {
  const result = await fetch(`resources/content-length.py?length=${encodeURIComponent("Content-Length: 50")}`);
  await promise_rejects_js(t, TypeError, result.text());
}, "Content-Length header value of network response exceeds response body");
