// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Substitution pattern: two-digit capturing group reference
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
    0x0024, N, N
    Where 0x0030 ≤ N ≤ 0x0039

    Unicode Characters:
    $nn where
    n is one of 0 1 2 3 4 5 6 7 8 9

    Replacement text:
    The nnth element of captures, where nn is a two-digit decimal number in the
    range 01 to 99. If nn≤m and the nnth element of captures is undefined, use
    the empty String instead. If nn is 00 or nn>m, no replacement is done.
features: [Symbol.replace]
---*/

assert.sameValue(
  /b(c)(z)?(.)/[Symbol.replace]('abcde', '[$01$02$03]'), 'a[cd]e'
);

assert.sameValue(
  /b(c)(z)?(.)/[Symbol.replace]('abcde', '[$01$02$03$04$00]'), 'a[cd$04$00]e'
);
