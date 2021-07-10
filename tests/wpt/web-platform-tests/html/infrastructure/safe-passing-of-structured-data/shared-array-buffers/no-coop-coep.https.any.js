// META: global=window,worker

test(() => {
  // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
  assert_equals(globalThis.SharedArrayBuffer, undefined);
  assert_false("SharedArrayBuffer" in globalThis);
}, "SharedArrayBuffer constructor does not exist without COOP+COEP");

test(() => {
  // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
  const sab = new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;
  const channel = new MessageChannel();
  assert_throws_dom("DataCloneError", () => channel.port1.postMessage(sab));
}, "SharedArrayBuffer over MessageChannel without COOP+COEP");

test(() => {
  // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
  const sab = new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;
  const channel = new BroadcastChannel("Is mir egal");
  assert_throws_dom("DataCloneError", () => channel.postMessage(sab));
}, "SharedArrayBuffer over BroadcastChannel without COOP+COEP");

if (self.GLOBAL.isWindow()) {
  test(() => {
    // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
    const sab = new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;
    assert_throws_dom("DataCloneError", () => self.postMessage(sab));
  }, "SharedArrayBuffer over postMessage() without COOP+COEP");
}

test(() => {
  assert_false(self.crossOriginIsolated);
}, "Bonus: self.crossOriginIsolated");
