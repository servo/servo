promise_test(() =>
  fetch("resources/echo-critical-hint.py")
      .then((r) => r.text())
      .then((r) => {
        assert_equals(r, "FAIL");
      })
, "Critical-CH");
