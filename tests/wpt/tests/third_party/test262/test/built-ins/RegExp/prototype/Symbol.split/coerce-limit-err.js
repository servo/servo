// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Behavior when error thrown while coercing `limit` argument
info: |
    [...]
    17. If limit is undefined, let lim be 253–1; else let lim be
        ToLength(limit).
    18. ReturnIfAbrupt(lim).
features: [Symbol.split]
---*/

var uncoercibleObj = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(TypeError, function() {
  /./[Symbol.split]('', Symbol.split);
});

assert.throws(Test262Error, function() {
  /./[Symbol.split]('', uncoercibleObj);
});
