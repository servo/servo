// META: script=resources/util.js

// This test requires a navigation with a non-safe (i.e. non-GET) HTTP
// response, which the Critical-CH spec says to ignore. The most
// "straight-forward" way to do this in JS is by making a form with an
// unsafe method (e.g. POST) method and submit it.
make_unsafe_method_test = (url, target) =>
  async (t) => {
    // Build the form DOM element
    var form = document.createElement("form");
    form.setAttribute("method", "post");
    form.setAttribute("action", url);
    form.setAttribute("target", target); //don't navigate away from the page running the test...
    document.body.appendChild(form);

    var popup_window = window.open("/common/blank.html", target);
    assert_not_equals(popup_window, null, "Popup windows not allowed?");
    t.add_cleanup(() => popup_window.close());

    var message = new Promise((resolve) => {
      window.addEventListener('message', (e) => resolve(e.data), {once: true});
    });

    await clear_client_hints();
    form.submit();
    assert_equals(await message, "FAIL");
  }

// The subtests run sequentially (promise_test) so that the Accept-CH header
// in the first response can't race with the second subtest's request.
promise_test(make_unsafe_method_test(ECHO_URL, "popup1"), "Critical-CH unsafe method");

promise_test(make_unsafe_method_test(ECHO_URL+"?multiple=true", "popup2"), "Critical-CH w/ multiple headers and unsafe method");
