// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Substitution pattern: text after match
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

    Code units: 0x0024, 0x0027

    Unicode Characters: $'

    Replacement text:
    If tailPos ≥ stringLength, the replacement is the empty String. Otherwise
    the replacement is the substring of str that starts at index tailPos and
    continues to the end of str.
features: [Symbol.replace]
---*/

assert.sameValue(/c/[Symbol.replace]('abc', '[$\']'), 'ab[]');
assert.sameValue(/b/[Symbol.replace]('abc', '[$\']'), 'a[c]c');
