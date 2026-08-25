// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error thrown during string coercion of first parameter
es6id: 21.2.5.6
info: |
    21.2.5.6 RegExp.prototype [ @@match ] ( string )

    [...]
    3. Let S be ToString(string)
    4. ReturnIfAbrupt(S).
features: [Symbol.match]
---*/

var str = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  /./[Symbol.match](str);
});
