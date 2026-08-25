// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: >
    Behavior when error thrown while accessing `index` property of match result
info: |
    [...]
    14. Return Get(result, "index").
features: [Symbol.search]
---*/

var poisonedIndex = {
  get index() {
    throw new Test262Error();
  }
};
var fakeRe = {
  exec: function() {
    return poisonedIndex;
  }
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.search].call(fakeRe);
});
