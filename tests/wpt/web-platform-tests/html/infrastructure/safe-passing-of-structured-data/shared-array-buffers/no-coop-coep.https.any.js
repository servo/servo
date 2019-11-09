test(() => {
  const sab = new SharedArrayBuffer();
  const channel = new MessageChannel();
  assert_throws("DataCloneError", () => channel.port1.postMessage(sab));
}, "SharedArrayBuffer over MessageChannel without COOP+COEP");

test(() => {
  const sab = new SharedArrayBuffer();
  const channel = new BroadcastChannel("Is mir egal");
  assert_throws("DataCloneError", () => channel.postMessage(sab));
}, "SharedArrayBuffer over BroadcastChannel without COOP+COEP");

if (self.GLOBAL.isWindow()) {
  test(() => {
    const sab = new SharedArrayBuffer();
    assert_throws("DataCloneError", () => self.postMessage(sab));
  }, "SharedArrayBuffer over postMessage() without COOP+COEP");
}

test(() => {
  assert_false(self.crossOriginIsolated);
}, "Bonus: self.crossOriginIsolated");
