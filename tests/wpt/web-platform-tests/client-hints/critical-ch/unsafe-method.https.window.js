async_test((t) => {
  // This test requires a navigation with a non-safe (i.e. non-GET) HTTP
  // response, which the Critical-CH spec says to ignore. The most
  // "straight-forward" way to do this in JS is by making a form with an
  // unsafe method (e.g. POST) method and submit it.

  // Build the form DOM element
  var form = document.createElement("form");
  form.setAttribute("method", "post");
  form.setAttribute("action", "resources/echo-critical-hint.py");
  form.setAttribute("target", "popup"); //don't navigate away from the page running the test...
  document.body.appendChild(form);

  var popup_window = window.open("/common/blank.html", "popup");
  assert_not_equals(popup_window, null, "Popup windows not allowed?");

  popup_window.addEventListener('message', (e) => {
    t.step(()=>{assert_equals(e.data, "FAIL")});
    t.done();
  });

  form.submit();
}, "Critical-CH unsafe method")
