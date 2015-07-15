/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

window.wrappedJSObject.timeout_multiplier = %(timeout_multiplier)d;
window.wrappedJSObject.explicit_timeout = %(explicit_timeout)d;

window.wrappedJSObject.addEventListener("message", function listener(event) {
    if (event.data.type != "complete") {
        return;
    }
    window.wrappedJSObject.removeEventListener("message", listener);
    clearTimeout(timer);
    var tests = event.data.tests;
    var status = event.data.status;
    marionetteScriptFinished({test:"%(url)s",
                              tests: tests,
                              status: status.status,
                              message: status.message,
                              stack: status.stack});
}, false);

window.wrappedJSObject.win = window.open("%(abs_url)s", "%(window_id)s");

var timer = null;
if (%(timeout)s) {
    timer = setTimeout(function() {
        log("Timeout fired");
        window.wrappedJSObject.win.timeout();
    }, %(timeout)s);
}
