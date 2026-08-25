// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when coercion of `lastIndex` attribute throws an error
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    7. Return RegExpBuiltinExec(R, S).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    7. If global is false and sticky is false, let lastIndex be 0.
    8. Else, let lastIndex be ? ToLength(? Get(R, "lastIndex")).
features: [Symbol.match]
---*/

var r = /./y;
r.lastIndex = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  r[Symbol.match]('');
});
