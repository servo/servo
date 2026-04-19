// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Return value after successful match
es6id: 21.2.5.6
info: |
    [...]
    5. Let global be ToBoolean(Get(rx, "global")).
    6. ReturnIfAbrupt(global).
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    7. Return RegExpBuiltinExec(R, S).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    20. Let A be ArrayCreate(n + 1).
    [...]
    24. Perform CreateDataProperty(A, "index", matchIndex).
    25. Perform CreateDataProperty(A, "input", S).
    26. Let matchedSubstr be the matched substring (i.e. the portion of S
        between offset lastIndex inclusive and offset e exclusive).
    27. Perform CreateDataProperty(A, "0", matchedSubstr).
    [...]
    29. Return A.
features: [Symbol.match]
---*/

var result = /b./[Symbol.match]('abcd');

assert(Array.isArray(result));
assert.sameValue(result.index, 1);
assert.sameValue(result.input, 'abcd');
assert.sameValue(result.length, 1);
assert.sameValue(result[0], 'bc');
