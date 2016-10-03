/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

var callback = arguments[arguments.length - 1];
window.timeout_multiplier = %(timeout_multiplier)d;

window.addEventListener("message", function f(event) {
  if (event.data.type != "complete") {
    return;
  }
  window.removeEventListener("message", f);

  var tests = event.data.tests;
  var status = event.data.status;

  var subtest_results = tests.map(function(x) {
    return [x.name, x.status, x.message, x.stack]
  });
  clearTimeout(timer);
  callback(["%(url)s",
            status.status,
            status.message,
            status.stack,
            subtest_results]);
}, false);

window.win = window.open("%(abs_url)s", "%(window_id)s");

var timer = setTimeout(function() {
  window.win.timeout();
  window.win.close();
}, %(timeout)s);
