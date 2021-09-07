// META: title=MessagePort message events are trusted

// See also:
// - https://github.com/whatwg/html/issues/1602
// - https://github.com/whatwg/html/pull/1935

"use strict";

async_test(t => {
  assert_true("MessageChannel" in self, "The browser must support MessageChannel");

  const channel = new MessageChannel();

  channel.port2.onmessage = t.step_func_done(e => {
    assert_equals(e.isTrusted, true);
  });

  channel.port1.postMessage("ping");
}, "With a MessageChannel and its MessagePorts");

async_test(t => {
  assert_true("BroadcastChannel" in self, "The browser must support BroadcastChannel");

  const channel = new BroadcastChannel("channel name");

  channel.onmessage = t.step_func_done(e => {
    assert_equals(e.isTrusted, true);
  });

  new Worker("support/MessageEvent-trusted-worker.js");
}, "With a BroadcastChannel");

