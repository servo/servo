// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.towellformed
description: >
  The method should coerce the receiver to a string.
info: |
  String.prototype.toWellFormed ( )

  1. Let O be ? RequireObjectCoercible(this value).
  2. Let S be ? ToString(O).
  …

features: [String.prototype.toWellFormed]
---*/

var obj = {
    toString: function () {
        throw new Test262Error('calls ToString');
    }
};

assert.throws(
    Test262Error,
    function () { String.prototype.toWellFormed.call(obj); },
    'coerces the receiver to a string'
);
