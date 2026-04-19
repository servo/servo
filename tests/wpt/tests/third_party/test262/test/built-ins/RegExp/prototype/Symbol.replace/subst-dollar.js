// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Substitution pattern: dollar sign
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

    Code units: 0x0024, 0x0024
    Unicode Characters: $$
    Replacement text: $

    [...]

    Code units: 0x0024
    Unicode Characters: $ in any context that does not match any of the above.
    Replacement text: $
features: [Symbol.replace]
---*/

assert.sameValue(/./[Symbol.replace]('abc', '$$'), '$bc', '"escaped" version');
assert.sameValue(
  /./[Symbol.replace]('abc', '$'), '$bc', '"unescaped" version'
);
assert.sameValue(
  /./[Symbol.replace]('abc', '\\$'), '\\$bc', 'backslash-prefixed'
);
assert.sameValue(
  /./[Symbol.replace]('abc', '$$$'),
  '$$bc',
  '"escaped" followed by "unuescaped"'
);
