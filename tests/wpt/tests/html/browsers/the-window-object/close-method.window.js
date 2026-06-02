function assert_closed_opener(w, closed, opener) {
  assert_equals(w.closed, closed);
  assert_equals(w.opener, opener);
}

async_test(t => {
  const openee = window.open();
  assert_closed_opener(openee, false, self);
  openee.onpagehide = t.step_func(() => {
    assert_closed_opener(openee, true, self);
    t.step_timeout(() => {
      assert_closed_opener(openee, true, null);
      t.done();
    }, 0);
  });
  openee.close();
  assert_closed_opener(openee, true, self);
}, "window.close() queues a task to discard, but window.closed knows immediately");

async_test(t => {
  const openee = window.open("", "greatname");
  assert_closed_opener(openee, false, self);
  openee.close();
  assert_closed_opener(openee, true, self);
  const openee2 = window.open("", "greatname");
  assert_not_equals(openee, openee2);
  assert_closed_opener(openee, true, self); // Ensure second window.open() call was synchronous
  openee2.onpagehide = t.step_func(() => {
    assert_closed_opener(openee2, true, self);
    t.step_timeout(() => {
      assert_closed_opener(openee, true, null);
      assert_closed_opener(openee2, true, null);
      t.done();
    }, 0);
  });
  openee2.close();
  assert_closed_opener(openee, true, self); // Ensure second close() call was synchronous
  assert_closed_opener(openee2, true, self);
}, "window.close() affects name targeting immediately");
