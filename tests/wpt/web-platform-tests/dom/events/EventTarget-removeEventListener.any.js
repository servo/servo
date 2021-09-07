// META: title=EventTarget.removeEventListener

// Step 1.
test(function() {
  assert_equals(document.removeEventListener("x", null, false), undefined);
  assert_equals(document.removeEventListener("x", null, true), undefined);
  assert_equals(document.removeEventListener("x", null), undefined);
}, "removing a null event listener should succeed");
