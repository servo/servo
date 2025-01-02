// META: title=XMLHttpRequest: abort() still works when error thrown internally
"use strict";

const test_runner = async_test();

test_runner.step(() => {
  const client = new XMLHttpRequest();

  client.open("GET", "invalid-protocol://example.com", true);
  client.onabort = test_runner.step_func(() => {
    test_runner.done();
  });

  client.send(null);
  client.abort();
});
