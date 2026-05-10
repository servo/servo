// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.iswellformed
description: >
  The method should coerce the receiver to a string.
info: |
  String.prototype.isWellFormed ( )

  1. Let O be ? RequireObjectCoercible(this value).
  2. Let S be ? ToString(O).
  3. Return IsStringWellFormedUnicode(S).

features: [String.prototype.isWellFormed]
---*/

var obj = {
    toString: function () {
        throw new Test262Error('calls ToString');
    }
};

assert.throws(
    Test262Error,
    function () { String.prototype.isWellFormed.call(obj); },
    'coerces the receiver to a string'
);
