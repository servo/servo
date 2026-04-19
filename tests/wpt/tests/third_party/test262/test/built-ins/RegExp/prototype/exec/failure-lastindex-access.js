// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: lastIndex is read but not written when global and sticky are unset.
es6id: 21.2.5.2.2
info: |
    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    4. Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
    [...]
    8. If global is false and sticky is false, let lastIndex be 0.
    [...]
    12. Repeat, while matchSucceeded is false
        [...]
        c. If r is failure, then
           i. If sticky is true, then
              1. Perform ? Set(R, "lastIndex", 0, true).
              2. Return null.
---*/

var gets = 0;
var counter = {
  valueOf: function() {
    gets++;
    return 0;
  }
};

var r = /a/;
r.lastIndex = counter;

var result = r.exec('nbc');
assert.sameValue(result, null);
assert.sameValue(r.lastIndex, counter);
assert.sameValue(gets, 1);
