// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    lastIndex is explicitly advanced for zero-length matches for "global"
    instances
es6id: 21.2.5.6
info: |
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          [...]
          iv. Else result is not null,
              [...]
              5. If matchStr is the empty String, then
                 [...]
                 c. Let nextIndex be AdvanceStringIndex(S, thisIndex,
                    fullUnicode).
                 d. Let setStatus be Set(rx, "lastIndex", nextIndex, true).
                 e. ReturnIfAbrupt(setStatus).
              6. Increment n.
features: [Symbol.match]
---*/

var result = /(?:)/g[Symbol.match]('abc');

assert.notSameValue(result, null);
assert.sameValue(result.length, 4);
assert.sameValue(result[0], '');
assert.sameValue(result[1], '');
assert.sameValue(result[2], '');
assert.sameValue(result[3], '');
