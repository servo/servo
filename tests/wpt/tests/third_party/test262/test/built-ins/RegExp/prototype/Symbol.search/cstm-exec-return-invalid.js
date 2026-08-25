// Copyright (C) 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Behavior when invalid value is returned by custom `exec` method
info: |
    [...]
    9. Let result be RegExpExec(rx, S).
    10. ReturnIfAbrupt(result).
    [...]
    14. Return Get(result, "index").

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    5. If IsCallable(exec) is true, then
       a. Let result be Call(exec, R, «S»).
       b. ReturnIfAbrupt(result).
       c. If Type(result) is neither Object or Null, throw a TypeError
          exception.

features: [Symbol, Symbol.search]
---*/

var retVal;
var fakeRe = {
  exec: function() {
    return retVal;
  }
};

retVal = undefined;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(fakeRe, 'a');
});

retVal = 86;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(fakeRe, 'a');
});

retVal = 'string';
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(fakeRe, 'a');
});

retVal = true;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(fakeRe, 'a');
});

retVal = Symbol();
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.search].call(fakeRe, 'a');
});
