importScripts("/resources/testharness.js");

test(function() {
  var entry = (new TestBinding()).entryGlobal();
  assert_equals(entry, self);
});


done();
