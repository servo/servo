// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Behavior when error thrown while executing match
info: |
    [...]
    9. Let result be RegExpExec(rx, S).
    10. ReturnIfAbrupt(result).
features: [Symbol.search]
---*/

var fakeRe = {
  lastIndex: 86,
  exec: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.search].call(fakeRe);
});

assert.sameValue(fakeRe.lastIndex, 0, '`lastIndex` property is not restored');
