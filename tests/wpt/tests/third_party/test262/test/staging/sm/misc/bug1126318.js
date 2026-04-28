// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

if (typeof window === "undefined") {
    var window = this;
}

Object.defineProperty(window, "foo", {
    get: function() { return 5; },
    configurable: true
});

for (var i = 0; i < 100; ++i)
    assert.sameValue(window.foo, 5);
