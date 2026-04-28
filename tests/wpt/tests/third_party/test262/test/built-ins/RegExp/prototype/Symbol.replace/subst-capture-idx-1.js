// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Substitution pattern: one-digit capturing group reference
es6id: 21.2.5.8
info: |
    16. Repeat, for each result in results,
        [...]
        m. If functionalReplace is true, then
           [...]
        n. Else,
           i. Let replacement be GetSubstitution(matched, S, position,
              captures, replaceValue).
        [...]

    21.1.3.14.1 Runtime Semantics: GetSubstitution

    Code units:
    0x0024, N
    Where 0x0031 ≤ N ≤ 0x0039

    Unicode Characters:
    $n where
    n is one of 1 2 3 4 5 6 7 8 9 and $n is not followed by a decimal digit

    Replacement text:
    The nth element of captures, where n is a single digit in the range 1 to 9.
    If n≤m and the nth element of captures is undefined, use the empty String
    instead. If n>m, no replacement is done.
features: [Symbol.replace]
---*/

assert.sameValue(/b(c)(z)?(.)/[Symbol.replace]('abcde', '[$1$2$3]'), 'a[cd]e');

assert.sameValue(/b(c)(z)?(.)/[Symbol.replace]('abcde', '[$1$2$3$4$0]'), 'a[cd$4$0]e');
