// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

if (typeof window === "undefined") {
    // This test is meant to run in the browser, but it's easy to
    // run it in the shell as well, even though it has no inner/outer
    // windows.
    var window = this;
}

var res = false;
Object.defineProperty(this, "foo", {configurable: true,
				    get: function() { return this === window; },
				    set: function(v) { res = this === window; }});

(function() {
    for (var i = 0; i < 3000; ++i) {
	window.foo = i;
	assert.sameValue(res, true, "setter");
	assert.sameValue(window.foo, true, "getter");
    }
})();
