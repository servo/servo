importScripts("/resources/testharness.js");
test(function() {
  assert_equals("�", "\ufffd");
}, "Decoding invalid utf-8");
done();
