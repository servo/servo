// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Encoding of `capturedValue`
es6id: 21.2.5.2.2
info: |
    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    12. Let flags be the value of R’s [[OriginalFlags]] internal slot.
    13. If flags contains "u", let fullUnicode be true, else let fullUnicode be
        false.
    [...]
    28. For each integer i such that i > 0 and i ≤ n
        a. Let captureI be ith element of r's captures List.
        b. If captureI is undefined, let capturedValue be undefined.
        c. Else if fullUnicode is true,
           i. Assert: captureI is a List of code points.
           ii. Let capturedValue be a string whose code units are the
               UTF16Encoding (10.1.1) of the code points of captureI.
        [...]
        e. Perform CreateDataProperty(A, ToString(i) , capturedValue).
    29. Return A.
---*/

var match = /./u.exec('𝌆');

assert(match !== null);
assert.sameValue(match.length, 1);
assert.sameValue(match[0].length, 2);
assert.sameValue(match[0][0], '\ud834');
assert.sameValue(match[0][1], '\udf06');
