// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

async_test(t => {
  addEventListener('message', t.step_func_done(e => {
    assert_equals(e.data, 'Allowed');
  }));
  const w = open("resources/page-with-top-navigating-iframe.html?user_gesture=true");
  t.add_cleanup(() => {w.close()});

}, "Cross-origin top navigation is allowed with user activation");
