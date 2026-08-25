// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// `var x` should not call the getter of an existing global property.

var hit = 0;
Object.defineProperty(this, "x", {
    get: function () { return ++hit; },
    configurable: true
});
eval("var x;");
assert.sameValue(hit, 0);

// The declaration should not have redefined the global x, either.
assert.sameValue(x, 1);
assert.sameValue(x, 2);

