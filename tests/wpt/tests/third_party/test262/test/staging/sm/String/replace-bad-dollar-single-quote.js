// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
let evil = new RegExp();

// https://tc39.es/ecma262/#sec-getsubstitution

// Input: position = 0
// Step 2: matchLength = 7
// Step 4: stringLength = 3
// Step 8: tailPos = position + matchLength = 7
//
// tailPos ≥ stringLength, so $' is replaced with the empty string.

evil.exec = () => ({ 0: "1234567", length: 1, index: 0 });
assert.sameValue("abc".replace(evil, "$'"), "");


// Input: position = 3
// Step 2: matchLength = 1
// Step 4: stringLength = 3
// Step 8: tailPos = position + matchLength = 4
//
// tailPos ≥ stringLength, so $' is replaced with the empty string.

evil.exec = () => ({ 0: "x", length: 1, index: 3 });
assert.sameValue("abc".replace(evil, "$'"), "abc");


// Input: position = 2
// Step 2: matchLength = 1
// Step 4: stringLength = 3
// Step 8: tailPos = position + matchLength = 3
//
// tailPos ≥ stringLength, so $' is replaced with the empty string.

evil.exec = () => ({ 0: "x", length: 1, index: 2 });
assert.sameValue("abc".replace(evil, "$'"), "ab");


// Input: position = 2
// Step 2: matchLength = 1
// Step 4: stringLength = 4
// Step 8: tailPos = position + matchLength = 3
//
// tailPos < stringLength, so $' is replaced with |"abcd".sustring(tailPos)| = "d".

evil.exec = () => ({ 0: "x", length: 1, index: 2 });
assert.sameValue("abcd".replace(evil, "$'"), "abdd");


// Input: position = 2
// Step 2: matchLength = 1
// Step 4: stringLength = 5
// Step 8: tailPos = position + matchLength = 3
//
// tailPos < stringLength, so $' is replaced with |"abcd".sustring(tailPos)| = "de".

evil.exec = () => ({ 0: "x", length: 1, index: 2 });
assert.sameValue("abcde".replace(evil, "$'"), "abdede");


