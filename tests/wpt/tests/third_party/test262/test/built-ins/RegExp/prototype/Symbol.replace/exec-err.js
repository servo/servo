// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when error is thrown by `exec` method
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    13. Repeat, while done is false
        a. Let result be RegExpExec(rx, S).
        b. ReturnIfAbrupt(result).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    3. Let exec be Get(R, "exec").
    4. ReturnIfAbrupt(exec).
    5. If IsCallable(exec) is true, then
       a. Let result be Call(exec, R, «S»).
       b. ReturnIfAbrupt(result).
features: [Symbol.replace]
---*/

var r = /./;
r.exec = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  r[Symbol.replace]('', '');
});
