// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: lastIndex is advanced according to width of astral symbols
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    10. If global is true, then
        a. Let fullUnicode be ToBoolean(Get(rx, "unicode")).
        b. ReturnIfAbrupt(fullUnicode).
    [...]
    13. Repeat, while done is false
        [...]
        d. Else result is not null,
           [...]
           iii. Else,
                [...]
                3. If matchStr is the empty String, then
                   [...]
                   c. Let nextIndex be AdvanceStringIndex(S, thisIndex,
                      fullUnicode).
                   d. Let setStatus be Set(rx, "lastIndex", nextIndex, true).
features: [Symbol.replace]
---*/

var str = /^|\udf06/ug[Symbol.replace]('\ud834\udf06', 'XXX');
assert.sameValue(str, 'XXX\ud834\udf06');
