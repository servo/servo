importScripts("/resources/testharness.js");

test(function() {
  assert_false("searchParams" in self.location,
               "location object should not have a searchParams attribute");
}, "searchParams on location object");

done();
