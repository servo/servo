test(() => {
  const closedPort = new MessageChannel().port1;
  closedPort.close();
  const { port1 } = new MessageChannel();
  closedPort.postMessage(0, [port1]);
  assert_throws_dom("DataCloneError", () => new MessageChannel().port1.postMessage(0, [port1]));
}, "Transferring a port from a closed port detaches it (transfer sequence)");

test(() => {
  const closedPort = new MessageChannel().port1;
  closedPort.close();
  const { port1 } = new MessageChannel();
  closedPort.postMessage(0, { transfer: [port1] });
  assert_throws_dom("DataCloneError", () => structuredClone(0, { transfer: [port1] }));
}, "Transferring a port from a closed port detaches it (StructuredSerializeOptions)");

test(() => {
  const closedPort = new MessageChannel().port1;
  closedPort.close();
  const { port1, port2 } = new MessageChannel();
  closedPort.postMessage(0, [port1, port2]);
  assert_throws_dom("DataCloneError", () => structuredClone(0, { transfer: [port1] }));
  assert_throws_dom("DataCloneError", () => structuredClone(0, { transfer: [port2] }));
}, "Transferring both ports of a channel from a closed port detaches both");

test(() => {
  const closedPort = new MessageChannel().port1;
  closedPort.close();
  const buffer = new ArrayBuffer(8);
  const { port1 } = new MessageChannel();
  closedPort.postMessage(0, [buffer, port1]);
  assert_equals(buffer.byteLength, 0);
  assert_throws_dom("DataCloneError", () => structuredClone(0, { transfer: [port1] }));
}, "Transferring a port and an ArrayBuffer from a closed port detaches both");

test(() => {
  const { port1, port2 } = new MessageChannel();
  port1.close();
  port1.postMessage(0, [port2]);
  assert_throws_dom("DataCloneError", () => structuredClone(0, { transfer: [port2] }));
}, "Transferring the formerly entangled port from a closed port detaches it");
