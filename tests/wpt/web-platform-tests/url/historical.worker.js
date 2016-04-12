importScripts("/resources/testharness.js");

test(function() {
  assert_false("searchParams" in self.location,
               "location object should not have a searchParams attribute");
}, "searchParams on location object");

test(function() {
  var url = new URL("./foo", "http://www.example.org");
  assert_equals(url.href, "http://www.example.org/foo");
  assert_throws(new TypeError(), function() {
    url.href = "./bar";
  });
}, "Setting URL's href attribute and base URLs");

done();
