// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var x = new ArrayBuffer(2);

var test = function(newProto) {
try {
    x.__proto__ = newProto;
    return false;
} catch(e) {
    return true;
}
}

// assert cycle doesn't work
assert.sameValue(test(x), true);

// works
assert.sameValue(test({}), false);
assert.sameValue(test(null), false);

