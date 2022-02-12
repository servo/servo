// Note: this function has a dependency on testdriver.js. Any test files calling
// it should include testdriver.js and testdriver-vendor.js
window.addEventListener("message", (e) => {
  let test_window = window.top;
  while (test_window.opener && !test_window.opener.closed) {
    test_window = test_window.opener.top;
  }
  test_driver.set_test_context(test_window);
  if (e.data == "getAndExpireCookiesForRedirectTest") {
    const cookies = document.cookie;
    test_driver.delete_all_cookies().then(() => {
      e.source.postMessage({"cookies": cookies}, '*');
    });
  }
});