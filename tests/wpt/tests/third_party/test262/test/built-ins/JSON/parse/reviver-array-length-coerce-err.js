// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-internalizejsonproperty
description: >
  Abrupt completion from coercing array "length" property to a number value
info: |
  JSON.parse ( text [ , reviver ] )

  [...]
  7. If IsCallable(reviver) is true, then
     [...]
     e. Return ? InternalizeJSONProperty(root, rootName).

  Runtime Semantics: InternalizeJSONProperty ( holder, name)

  1. Let val be ? Get(holder, name).
  2. If Type(val) is Object, then
     a. Let isArray be ? IsArray(val).
     b. If isArray is true, then
        i. Set I to 0.
        ii. Let len be ? ToLength(? Get(val, "length")).
features: [Proxy]
---*/

var uncoercible = {
  valueOf: function() {
    throw new Test262Error();
  }
};
var badLength = new Proxy([], {
  get: function(_, name) {
    if (name === 'length') {
      return uncoercible;
    }
  }
});

assert.throws(Test262Error, function() {
  JSON.parse('[0,0]', function() {
    this[1] = badLength;
  });
});
