// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when error is thrown while accessing `exec` property
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    3. Let exec be Get(R, "exec").
    4. ReturnIfAbrupt(exec).
features: [Symbol.match]
---*/

var r = /./;

Object.defineProperty(r, 'exec', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  r[Symbol.match]('');
});
