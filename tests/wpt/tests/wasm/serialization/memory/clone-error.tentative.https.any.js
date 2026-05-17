// META: global=window,worker

"use strict";

test(() => {
  const memory = new WebAssembly.Memory({ initial: 1 });
  assert_throws_dom("DataCloneError", () => structuredClone(memory));
}, "structuredClone() of a non-shared WebAssembly.Memory throws");

test(() => {
  const memory = new WebAssembly.Memory({ initial: 1 });
  const channel = new MessageChannel();
  assert_throws_dom("DataCloneError", () => channel.port1.postMessage(memory));
}, "Cloning a non-shared WebAssembly.Memory through a MessagePort throws");

test(() => {
  const memory = new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 });
  const channel = new MessageChannel();
  assert_throws_dom("DataCloneError", () => channel.port1.postMessage(memory));
}, "Cloning a shared WebAssembly.Memory through a MessagePort without COOP+COEP throws");

test(() => {
  const memory = new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 });
  const channel = new BroadcastChannel("Is mir egal");
  assert_throws_dom("DataCloneError", () => channel.postMessage(memory));
}, "Cloning a shared WebAssembly.Memory through a BroadcastChannel without COOP+COEP throws");

if (self.GLOBAL.isWindow()) {
  test(() => {
    const memory = new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 });
    assert_throws_dom("DataCloneError", () => self.postMessage(memory));
  }, "Cloning a shared WebAssembly.Memory via self.postMessage() without COOP+COEP throws");
}

test(() => {
  assert_false(self.crossOriginIsolated);
}, "Bonus: self.crossOriginIsolated is false (precondition for the shared-memory cases above)");
