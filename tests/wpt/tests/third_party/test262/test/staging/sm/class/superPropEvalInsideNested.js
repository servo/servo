// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// It's invalid to eval super.prop inside a nested non-method, even if it
// appears inside a method definition
assert.throws(SyntaxError, () =>
({
    method() {
        (function () {
            eval('super.toString');
        })();
    }
}).method());

