/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

var callback = arguments[arguments.length - 1];

function test(x) {
  if (!root.classList.contains("reftest-wait")) {
    observer.disconnect();
    callback()
  }
}

var root = document.documentElement;
var observer = new MutationObserver(test);

observer.observe(root, {attributes: true});

if (document.readyState != "complete") {
    onload = test;
} else {
    test();
}
