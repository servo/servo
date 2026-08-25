// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Behavior when error thrown while coercing `string` argument
info: |
    [...]
    3. Let S be ToString(string).
    4. ReturnIfAbrupt(S).
features: [Symbol.search]
---*/

var uncoercibleObj = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  /./[Symbol.search](uncoercibleObj);
});

assert.throws(TypeError, function() {
  /./[Symbol.search](Symbol.search);
});
