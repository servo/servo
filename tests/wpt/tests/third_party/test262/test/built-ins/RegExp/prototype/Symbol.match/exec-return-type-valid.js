// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when `exec` method returns value of valid type
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
       d. Return result.
features: [Symbol.match]
---*/

var r = /./;
var retValue;
r.exec = function() {
  return retValue;
};

retValue = null;
assert.sameValue(r[Symbol.match](''), retValue);

retValue = {};
assert.sameValue(r[Symbol.match](''), retValue);
