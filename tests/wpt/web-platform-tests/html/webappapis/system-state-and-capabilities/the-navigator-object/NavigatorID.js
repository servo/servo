function run_test() {
  test(function() {
    assert_equals(navigator.appCodeName, "Mozilla");
  }, "appCodeName");

  test(function() {
    assert_equals(typeof navigator.appName, "string",
                  "navigator.appName should be a string");
  }, "appName");

  test(function() {
    assert_equals(typeof navigator.appVersion, "string",
                  "navigator.appVersion should be a string");
  }, "appVersion");

  test(function() {
    assert_equals(typeof navigator.platform, "string",
                  "navigator.platform should be a string");
  }, "platform");

  test(function() {
    assert_equals(navigator.product, "Gecko");
  }, "product");

  test(function() {
    // See https://www.w3.org/Bugs/Public/show_bug.cgi?id=22555
    if ("window" in self) {
      // If you identify as WebKit, taintEnabled should not exist.
      if (navigator.userAgent.indexOf("WebKit") != -1) {
        assert_false("taintEnabled" in navigator);
      }
      // Otherwise it should exist and return false.
      else {
        assert_false(navigator.taintEnabled());
      }
    } else {
      // taintEnabled should not exist in workers.
      assert_false("taintEnabled" in navigator);
    }
  }, "taintEnabled");

  test(function() {
    assert_equals(typeof navigator.userAgent, "string",
                  "navigator.userAgent should be a string");
  }, "userAgent type");

  test(function() {
    assert_equals(navigator.vendorSub, "");
  }, "vendorSub");

  async_test(function() {
    var request = new XMLHttpRequest();
    request.onload = this.step_func_done(function() {
      assert_equals("user-agent: " + navigator.userAgent + "\n",
                    request.response,
                    "userAgent should return the value sent in the " +
                    "User-Agent header");
    });
    request.open("GET", "/XMLHttpRequest/resources/inspect-headers.py?" +
                        "filter_name=User-Agent");
    request.send();
  }, "userAgent value");
}
