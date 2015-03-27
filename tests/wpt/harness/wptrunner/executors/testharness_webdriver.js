/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

var callback = arguments[arguments.length - 1];
window.timeout_multiplier = %(timeout_multiplier)d;

window.done = function(tests, status) {
  clearTimeout(timer);
  var test_results = tests.map(function(x) {
    return {name:x.name, status:x.status, message:x.message, stack:x.stack}
  });
  callback({test:"%(url)s",
            tests:test_results,
            status: status.status,
            message: status.message,
            stack: status.stack});
}

window.win = window.open("%(abs_url)s", "%(window_id)s");

var timer = setTimeout(function() {
  window.win.timeout();
  window.win.close();
}, %(timeout)s);
