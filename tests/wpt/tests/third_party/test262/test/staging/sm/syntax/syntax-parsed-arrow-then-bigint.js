/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Properly evaluate a bigint literal that's initially tokenized by a syntax parser (because the bigint literal appears immediately after an arrow function with expression body)
info: bugzilla.mozilla.org/show_bug.cgi?id=1596706
esid: pending
---*/

// block followed by semicolon
assert.sameValue(eval(`x=>{};
17n`), 17n);

// block not followed by semicolon
assert.sameValue(eval(`x=>{}
42n`), 42n);

// expr followed by semicolon
assert.sameValue(eval(`x=>y;
8675309n`), 8675309n);

// expr not followed by semicolon
assert.sameValue(eval(`x=>y
78051120n`), 78051120n);
