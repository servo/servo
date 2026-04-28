// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: lastIndex is accessed when global is set.
es6id: 21.2.5.2.2
info: |
    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let flags be R.[[OriginalFlags]].
    5. If flags contains "g", let global be true, else let global be false.
    [...]
    15. If global is true or sticky is true, then
        a. Perform ? Set(R, "lastIndex", e, true).
---*/

var lastIndexReads = 0;

var r = /./g;
r.lastIndex = {
  valueOf: function() {
    lastIndexReads++;
    return 0;
  }
};

var result = r.exec('abc');
assert.sameValue(result.length, 1);
assert.sameValue(result[0], 'a');
assert.sameValue(r.lastIndex, 1);
assert.sameValue(lastIndexReads, 1);

