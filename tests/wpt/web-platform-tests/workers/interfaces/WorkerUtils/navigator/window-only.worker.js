importScripts("/resources/testharness.js");

var properties = [
  "appCodeName",
  "product",
  "productSub",
  "vendor",
  "vendorSub",

  // Only exist in Window scopes if navigator compatibility mode is Gecko;
  // never exist in workers.
  "taintEnabled",
  "oscpu",
];

properties.forEach(function(property) {
  test(function() {
    assert_false(property in navigator);
  }, "NavigatorID properties exposed only for Window: " + property);
});

done();
