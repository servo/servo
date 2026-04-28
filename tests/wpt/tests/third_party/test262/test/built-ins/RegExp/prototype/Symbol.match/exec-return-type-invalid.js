// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when `exec` method returns value of invalid type
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    5. If IsCallable(exec) is true, then
       a. Let result be Call(exec, R, «S»).
       b. ReturnIfAbrupt(result).
       c. If Type(result) is neither Object or Null, throw a TypeError
          exception.
features: [Symbol.match]
---*/

var r = /./;
var retValue;
r.exec = function() {
  return retValue;
};

// Explicitly assert the method's presence to avoid false positives (i.e.
// TypeErrors thrown by invoking an undefined reference).
assert.sameValue(typeof r[Symbol.match], 'function');

retValue = undefined;
assert.throws(TypeError, function() {
  r[Symbol.match]('');
});

retValue = true;
assert.throws(TypeError, function() {
  r[Symbol.match]('');
});

retValue = 'string';
assert.throws(TypeError, function() {
  r[Symbol.match]('');
});

retValue = Symbol.match;
assert.throws(TypeError, function() {
  r[Symbol.match]('');
});

retValue = 86;
assert.throws(TypeError, function() {
  r[Symbol.match]('');
});
