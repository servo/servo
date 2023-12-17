// META: title=Close event test when an entangled port is GCed.
// META: script=/common/gc.js

/**
 * Create a new MessageChannel and return port1 and a weak reference to port2.
 * It is expected that port2 will be garbage collected and a close event
 * will be fired on port1.
 *
 * @returns {Array.<[MessagePort, WeakRef<MessagePort>]>}
 */
function createMessageChannelAndWeakReferToPort() {
  const {port1, port2} = new MessageChannel();
  port1.start();
  return [port1, new WeakRef(port2)];
}

promise_test(async t => {
  const [port1, weakport2] = createMessageChannelAndWeakReferToPort();
  const closeEventPromise = new Promise(resolve => port1.onclose = resolve);
  garbageCollect();
  await closeEventPromise;
  assert_equals(weakport2.deref(), undefined, 'port2 should be GCed');
}, 'Entangled port is garbage collected, and the close event is fired.')
