// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when there is an error thrown while accessing the `exec` method of
    "global" instances
es6id: 21.2.5.6
info: |
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          ii. ReturnIfAbrupt(result).

    ES6 21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    3. Let exec be Get(R, "exec").
    4. ReturnIfAbrupt(exec).
features: [Symbol.match]
---*/

var r = { flags: 'g', global: true };
Object.defineProperty(r, 'exec', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.match].call(r, '');
});

assert.sameValue(r.lastIndex, 0, 'Error thrown after setting `lastIndex`');
