// META: title=postMessage() transferring the target port

test(() => {
  const channel = new MessageChannel();
  channel.port1.postMessage("doomed", [channel.port2]);
}, "Transferring the entangled target port does not throw");

test(() => {
  const channel = new MessageChannel();
  channel.port1.postMessage("doomed", [channel.port2]);
  assert_throws_dom("DataCloneError", () => {
    structuredClone(null, { transfer: [channel.port2] });
  });
}, "Transferring the entangled target port detaches it");

test(() => {
  const channel = new MessageChannel();
  channel.port1.postMessage("doomed", { transfer: [channel.port2] });
  assert_throws_dom("DataCloneError", () => {
    structuredClone(null, { transfer: [channel.port2] });
  });
}, "Transferring the entangled target port via StructuredSerializeOptions detaches it");
