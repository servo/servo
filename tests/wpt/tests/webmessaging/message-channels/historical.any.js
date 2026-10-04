// META: global=window,worker

// How long (in ms) these tests should wait before deciding no close event
// will be fired.
const time_to_wait_for_events = 100;

test(() => {
  assert_false("onclose" in MessagePort.prototype, "MessagePort.prototype");
  const c = new MessageChannel();
  assert_false("onclose" in c.port1, "MessagePort instance");
}, "MessagePort should not have an onclose event handler IDL attribute");

async_test(t => {
  const c = new MessageChannel();
  c.port1.addEventListener("close", t.unreached_func("close event fired on port1"));
  c.port2.addEventListener("close", t.unreached_func("close event fired on port2"));
  c.port1.start();
  c.port2.start();
  c.port2.close();
  t.step_timeout(() => t.done(), time_to_wait_for_events);
}, "No close event should be fired when an entangled port is closed");

async_test(t => {
  const c = new MessageChannel();
  c.port1.onmessage = () => {};
  c.port1.addEventListener("close", t.unreached_func("close event fired on port1"));
  c.port1.close();
  t.step_timeout(() => t.done(), time_to_wait_for_events);
}, "No close event should be fired on a port when it is closed");
