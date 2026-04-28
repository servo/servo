// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Basic matching cases with non-Unicode groups
esid: prod-GroupSpecifier
features: [regexp-named-groups]
includes: [compareArray.js]
---*/

assert.compareArray(["a", "a"], "bab".match(/(?<a>a)/));
assert.compareArray(["a", "a"], "bab".match(/(?<a42>a)/));
assert.compareArray(["a", "a"], "bab".match(/(?<_>a)/));
assert.compareArray(["a", "a"], "bab".match(/(?<$>a)/));
assert.compareArray(["bab", "a"], "bab".match(/.(?<$>a)./));
assert.compareArray(["bab", "a", "b"], "bab".match(/.(?<a>a)(.)/));
assert.compareArray(["bab", "a", "b"], "bab".match(/.(?<a>a)(?<b>.)/));
assert.compareArray(["bab", "ab"], "bab".match(/.(?<a>\w\w)/));
assert.compareArray(["bab", "bab"], "bab".match(/(?<a>\w\w\w)/));
assert.compareArray(["bab", "ba", "b"], "bab".match(/(?<a>\w\w)(?<b>\w)/));

let {a, b, c} = /(?<a>.)(?<b>.)(?<c>.)\k<c>\k<b>\k<a>/.exec("abccba").groups;
assert.sameValue(a, "a");
assert.sameValue(b, "b");
assert.sameValue(c, "c");

assert.compareArray("bab".match(/(a)/), "bab".match(/(?<a>a)/));
assert.compareArray("bab".match(/(a)/), "bab".match(/(?<a42>a)/));
assert.compareArray("bab".match(/(a)/), "bab".match(/(?<_>a)/));
assert.compareArray("bab".match(/(a)/), "bab".match(/(?<$>a)/));
assert.compareArray("bab".match(/.(a)./), "bab".match(/.(?<$>a)./));
assert.compareArray("bab".match(/.(a)(.)/), "bab".match(/.(?<a>a)(.)/));
assert.compareArray("bab".match(/.(a)(.)/), "bab".match(/.(?<a>a)(?<b>.)/));
assert.compareArray("bab".match(/.(\w\w)/), "bab".match(/.(?<a>\w\w)/));
assert.compareArray("bab".match(/(\w\w\w)/), "bab".match(/(?<a>\w\w\w)/));
assert.compareArray("bab".match(/(\w\w)(\w)/), "bab".match(/(?<a>\w\w)(?<b>\w)/));

assert.compareArray(["bab", "b"], "bab".match(/(?<b>b).\1/));
assert.compareArray(["baba", "b", "a"], "baba".match(/(.)(?<a>a)\1\2/));
assert.compareArray(["baba", "b", "a", "b", "a"], "baba".match(/(.)(?<a>a)(?<b>\1)(\2)/));
assert.compareArray(["<a", "<"], "<a".match(/(?<lt><)a/));
assert.compareArray([">a", ">"], ">a".match(/(?<gt>>)a/));
