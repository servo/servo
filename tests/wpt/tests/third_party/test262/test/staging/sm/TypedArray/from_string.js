// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// %TypedArray%.from called on Array should also handle strings correctly.
var from = Int8Array.from.bind(Uint32Array);
var toCodePoint = s => s.codePointAt(0);

// %TypedArray%.from on a string iterates over the string.
assert.compareArray(from("test string", toCodePoint),
             ['t', 'e', 's', 't', ' ', 's', 't', 'r', 'i', 'n', 'g'].map(toCodePoint));

// %TypedArray%.from on a string handles surrogate pairs correctly.
var gclef = "\uD834\uDD1E"; // U+1D11E MUSICAL SYMBOL G CLEF
assert.compareArray(from(gclef, toCodePoint), [gclef].map(toCodePoint));
assert.compareArray(from(gclef + " G", toCodePoint), [gclef, " ", "G"].map(toCodePoint));

// %TypedArray%.from on a string calls the @@iterator method.
String.prototype[Symbol.iterator] = function* () { yield 1; yield 2; };
assert.compareArray(from("anything"), [1, 2]);

// If the iterator method is deleted, Strings are still arraylike.
delete String.prototype[Symbol.iterator];
assert.compareArray(from("works", toCodePoint), ['w', 'o', 'r', 'k', 's'].map(toCodePoint));
assert.compareArray(from(gclef, toCodePoint), ['\uD834', '\uDD1E'].map(toCodePoint));

