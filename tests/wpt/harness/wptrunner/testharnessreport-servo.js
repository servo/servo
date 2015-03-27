/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

var props = {output:%(output)d};

setup(props);

add_completion_callback(function (tests, harness_status) {
    alert("RESULT: " + JSON.stringify({
        tests: tests.map(function(t) {
            return { name: t.name, status: t.status, message: t.message, stack: t.stack}
        }),
        status: harness_status.status,
        message: harness_status.message,
        stack: harness_status.stack,
    }));
});
