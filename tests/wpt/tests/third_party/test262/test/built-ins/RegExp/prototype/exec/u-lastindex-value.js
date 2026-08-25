// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Definition of `lastIndex` property value
es6id: 21.2.5.2.2
info: |
    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    12. Let flags be the value of R’s [[OriginalFlags]] internal slot.
    13. If flags contains "u", let fullUnicode be true, else let fullUnicode be
        false.
    [...]
    16. Let e be r's endIndex value.
    17. If fullUnicode is true, then
        a. e is an index into the Input character list, derived from S, matched
           by matcher. Let eUTF be the smallest index into S that corresponds
           to the character at element e of Input. If e is greater than or
           equal to the length of Input, then eUTF is the number of code units
           in S.
        b. Let e be eUTF.
    18. If global is true or sticky is true,
        a. Let setStatus be Set(R, "lastIndex", e, true).
        b. ReturnIfAbrupt(setStatus).
---*/

var r = /./ug;
r.exec('𝌆');
assert.sameValue(r.lastIndex, 2);
