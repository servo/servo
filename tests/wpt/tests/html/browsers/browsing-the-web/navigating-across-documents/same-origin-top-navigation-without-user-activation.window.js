async_test(t => {
  addEventListener('message', t.step_func_done(e => {
    assert_equals(e.data, 'Allowed');
  }));

  const w = open("resources/page-with-top-navigating-iframe.html?same_origin=true");
  t.add_cleanup(() => {w.close()});

}, "Same-origin top navigation is allowed without user activation");
