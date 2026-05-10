// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype-@@replace
description: >
  Both functional and pattern replacement performs as expected with poisoned stdlib.
features: [Symbol.iterator, Symbol.replace, regexp-named-groups]
---*/

assert(delete Array.prototype.concat);
assert(delete Array.prototype.push);
assert(delete Array.prototype[Symbol.iterator]);
assert(delete Function.prototype.apply);
assert(delete String.prototype.charAt);
assert(delete String.prototype.charCodeAt);
assert(delete String.prototype.indexOf);
assert(delete String.prototype.slice);
assert(delete String.prototype.substring);

for (let i = 0; i < 5; ++i) {
    Object.defineProperty(Array.prototype, i, {
        get: function() {
            throw new Test262Error(i + " getter should be unreachable.");
        },
        set: function(_value) {
            throw new Test262Error(i + " setter should be unreachable.");
        },
    });
}

var str = "1a2";

assert.sameValue(/a/[Symbol.replace](str, "$`b"), "11b2");
assert.sameValue(/a/[Symbol.replace](str, "b$'"), "1b22");
assert.sameValue(/a/[Symbol.replace](str, "$3b$33"), "1$3b$332");
assert.sameValue(/(a)/[Symbol.replace](str, "$1b"), "1ab2");
assert.sameValue(/(?<a>a)/[Symbol.replace](str, "$<a>b"), "1ab2");

var replacer = function() {
  return "b";
};

assert.sameValue(/a/[Symbol.replace](str, replacer), "1b2");
