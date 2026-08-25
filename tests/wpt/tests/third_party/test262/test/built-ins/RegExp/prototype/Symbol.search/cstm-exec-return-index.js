// Copyright (C) 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Index value returned by a custom `exec` method
info: |
    [...]
    9. Let result be RegExpExec(rx, S).
    [...]
    14. Return Get(result, "index").

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    5. If IsCallable(exec) is true, then
       a. Let result be Call(exec, R, «S»).
       b. ReturnIfAbrupt(result).
       c. If Type(result) is neither Object or Null, throw a TypeError
          exception.
       d. Return result.

features: [Symbol.search]
---*/

var fakeRe = {
  exec: function() {
    return { index: 86 };
  }
};

assert.sameValue(RegExp.prototype[Symbol.search].call(fakeRe, 'abc'), 86);
