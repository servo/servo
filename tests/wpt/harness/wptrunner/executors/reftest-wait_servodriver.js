/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

callback = arguments[arguments.length - 1];

function check_done() {
    if (!document.body.classList.contains('reftest-wait')) {
        callback();
    } else {
        setTimeout(check_done, 50);
    }
}

if (document.readyState === 'complete') {
    check_done();
} else {
    addEventListener("load", check_done);
}
