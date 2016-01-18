t.step(function() {
  assert_false(dcl, "DOMContentLoaded should not have fired before executing " +
                    "a defer script");

  setTimeout(t.step_func(function() {
    assert_false(dcl, "DOMContentLoaded should not have fired before " +
                      "executing a task queued from a defer script");
    setTimeout(t.step_func_done(function() {
      assert_true(dcl, "DOMContentLoaded should have fired in a task that " +
                       "was queued after the DOMContentLoaded task was queued");
    }), 0);
  }), 0);
});
