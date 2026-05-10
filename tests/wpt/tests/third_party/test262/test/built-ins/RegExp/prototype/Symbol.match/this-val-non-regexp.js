// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when invoked on an object without a a [[RegExpMatcher]] internal
    slot
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    5. If IsCallable(exec) is true, then
       [...]
       d. Return result.
    6. If R does not have a [[RegExpMatcher]] internal slot, throw a TypeError
       exception.
features: [Symbol.match]
---*/

var objWithExec = {
  exec: function() {
    return null;
  }
};
var objWithoutExec = {};

RegExp.prototype[Symbol.match].call(objWithExec);

assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.match].call(objWithoutExec);
});
