async_test(t => {
  addEventListener('message', t.step_func_done(e => {
    assert_equals(e.data, 'Denied');
  }));
  const w = open("resources/page-with-nested-top-navigating-iframe.sub.html");
  t.add_cleanup(() => {w.close()});

}, "Cross-origin top navigation is blocked without user activation, even if iframe is nested and its direct parent is same-site");
