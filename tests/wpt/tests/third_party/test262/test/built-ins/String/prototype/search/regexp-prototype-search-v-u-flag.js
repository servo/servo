// Copyright (C) 2024 Tan Meng. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype-@@search
description: RegExp.prototype[@@search] behavior with 'v' flag, comparing with 'u' flag
features: [Symbol.search, regexp-v-flag]
---*/

const text = '𠮷a𠮷b𠮷c👨‍👩‍👧‍👦d';

function doSearch(regex) {
  return RegExp.prototype[Symbol.search].call(regex, text);
}

assert.sameValue(doSearch(/a/), 2, "Basic search without flags");
assert.sameValue(doSearch(/a/u), 2, "Search with u flag");
assert.sameValue(doSearch(/a/v), 2, "Search with v flag");

// Surrogate pair search
assert.sameValue(doSearch(/𠮷/), 0, "Search for surrogate pair without flags");
assert.sameValue(doSearch(/𠮷/u), 0, "Search for surrogate pair with u flag");
assert.sameValue(doSearch(/𠮷/v), 0, "Search for surrogate pair with v flag");

// Unicode property escapes
assert.sameValue(doSearch(/\p{Script=Han}/u), 0, "Unicode property escapes with u flag");
assert.sameValue(doSearch(/\p{Script=Han}/v), 0, "Unicode property escapes with v flag");

// Dot behavior
assert.sameValue(doSearch(/c./), 8, "Dot without u or v flag");
assert.sameValue(doSearch(/c./u), 8, "Dot with u flag");
assert.sameValue(doSearch(/c./v), 8, "Dot with v flag");

// Complex emoji sequence
assert.sameValue(doSearch(/👨‍👩‍👧‍👦/u), 9, "Complex emoji sequence with u flag");
assert.sameValue(doSearch(/👨‍👩‍👧‍👦/v), 9, "Complex emoji sequence with v flag");

// Set notation
assert.sameValue(doSearch(/[👨‍👩‍👧‍👦]/v), 9, "Complex emoji sequence in set notation with v flag");
assert.sameValue(doSearch(/[👨‍👩‍👧‍👦]/u), 9, "Complex emoji sequence in set notation with u flag throws");

// Non-existent pattern
assert.sameValue(doSearch(/x/u), -1, "Search for non-existent pattern with u flag");
assert.sameValue(doSearch(/x/v), -1, "Search for non-existent pattern with v flag");
