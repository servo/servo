// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Return original string when no matches occur
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    12. Let done be false.
    13. Repeat, while done is false
        a. Let result be RegExpExec(rx, S).
        b. ReturnIfAbrupt(result).
        c. If result is null, set done to true.
        [...]
    14. Let accumulatedResult be the empty String value.
    15. Let nextSourcePosition be 0.
    [...]
    18. Return the String formed by concatenating the code units of
        accumulatedResult with the substring of S consisting of the code units
        from nextSourcePosition (inclusive) up through the final code unit of S
        (inclusive).
features: [Symbol.replace]
---*/

assert.sameValue(/x/[Symbol.replace]('abc', 'X'), 'abc');
