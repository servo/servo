// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: lastIndex read but not written when global and sticky are unset.
es6id: 21.2.5.2.2
info: |
    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
    5. Let flags be R.[[OriginalFlags]].
    6. If flags contains "g", let global be true, else let global be false.
    [...]
    15. If global is true or sticky is true, then
        a. Perform ? Set(R, "lastIndex", e, true).
---*/

var gets = 0;
var counter = {
  valueOf: function() {
    gets++;
    return 0;
  }
};

var r = /./;
r.lastIndex = counter;

var result = r.exec('abc');

assert.notSameValue(result, null);
assert.sameValue(result.length, 1);
assert.sameValue(result[0], 'a');
assert.sameValue(r.lastIndex, counter);
assert.sameValue(gets, 1);
