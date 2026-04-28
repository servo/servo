// Copyright (C) 2024 Tan Meng. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype-@@search
description: RegExp.prototype[@@search] behavior with 'v' flag
features: [Symbol.search, regexp-v-flag]
---*/

const text = '𠮷a𠮷b𠮷';

function doSearch(regex) {
  return RegExp.prototype[Symbol.search].call(regex, text);
}

assert.sameValue(doSearch(/a/), 2, "Basic search without v flag");
assert.sameValue(doSearch(/a/v), 2, "Search with v flag");
assert.sameValue(doSearch(/𠮷/), 0, "Search for surrogate pair without v flag");
assert.sameValue(doSearch(/𠮷/v), 0, "Search for surrogate pair with v flag");
assert.sameValue(doSearch(/\p{Script=Han}/v), 0, "Unicode property escapes with v flag");
assert.sameValue(doSearch(/b./v), 5, "Dot with v flag");
