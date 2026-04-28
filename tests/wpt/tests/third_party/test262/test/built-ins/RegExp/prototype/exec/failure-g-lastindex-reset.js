// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    lastIndex is read and reset to 0 when global is set and the match fails.
es6id: 21.2.5.2.2
info: |
    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let flags be R.[[OriginalFlags]].
    5. If flags contains "g", let global be true, else let global be false.
    [...]
    7. If global is false and sticky is false, let lastIndex be 0.
    8. Else, let lastIndex be ? ToLength(? Get(R, "lastIndex")).
    [...]
    12. Repeat, while matchSucceeded is false
        [...]
        c. If r is failure, then
           i. If sticky is true, then
              1. Perform ? Set(R, "lastIndex", 0, true).
              2. Return null.
           ii. Let lastIndex be AdvanceStringIndex(S, lastIndex, fullUnicode).
---*/

var lastIndexReads;
var result;

var r = /a/g;

function reset(value) {
  r.lastIndex = {
    valueOf: function() {
      lastIndexReads++;
      return value;
    }
  };
  lastIndexReads = 0;
}

reset(42);  // lastIndex beyond end of string.
result = r.exec('abc');
assert.sameValue(result, null);
assert.sameValue(r.lastIndex, 0);
assert.sameValue(lastIndexReads, 1);

reset(-1);  // No match.
result = r.exec('nbc');
assert.sameValue(result, null);
assert.sameValue(r.lastIndex, 0);
assert.sameValue(lastIndexReads, 1);
