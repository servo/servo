// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function f(x) {
    Object.defineProperty(arguments, 0, {
        get: function() {}
    });
    return arguments;
}

var obj = f(1);
assert.sameValue(obj[0], undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(obj, 0).set, undefined);
assert.throws(TypeError, () => { "use strict"; obj[0] = 1; });

