// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Basic matching cases with Unicode groups
esid: prod-GroupSpecifier
features: [regexp-named-groups]
includes: [compareArray.js]
---*/

assert.compareArray(["a", "a"], "bab".match(/(?<a>a)/u));
assert.compareArray(["a", "a"], "bab".match(/(?<a42>a)/u));
assert.compareArray(["a", "a"], "bab".match(/(?<_>a)/u));
assert.compareArray(["a", "a"], "bab".match(/(?<$>a)/u));
assert.compareArray(["bab", "a"], "bab".match(/.(?<$>a)./u));
assert.compareArray(["bab", "a", "b"], "bab".match(/.(?<a>a)(.)/u));
assert.compareArray(["bab", "a", "b"], "bab".match(/.(?<a>a)(?<b>.)/u));
assert.compareArray(["bab", "ab"], "bab".match(/.(?<a>\w\w)/u));
assert.compareArray(["bab", "bab"], "bab".match(/(?<a>\w\w\w)/u));
assert.compareArray(["bab", "ba", "b"], "bab".match(/(?<a>\w\w)(?<b>\w)/u));

let {a, b, c} = /(?<a>.)(?<b>.)(?<c>.)\k<c>\k<b>\k<a>/u.exec("abccba").groups;
assert.sameValue(a, "a");
assert.sameValue(b, "b");
assert.sameValue(c, "c");

assert.compareArray("bab".match(/(a)/u), "bab".match(/(?<a>a)/u));
assert.compareArray("bab".match(/(a)/u), "bab".match(/(?<a42>a)/u));
assert.compareArray("bab".match(/(a)/u), "bab".match(/(?<_>a)/u));
assert.compareArray("bab".match(/(a)/u), "bab".match(/(?<$>a)/u));
assert.compareArray("bab".match(/.(a)./u), "bab".match(/.(?<$>a)./u));
assert.compareArray("bab".match(/.(a)(.)/u), "bab".match(/.(?<a>a)(.)/u));
assert.compareArray("bab".match(/.(a)(.)/u), "bab".match(/.(?<a>a)(?<b>.)/u));
assert.compareArray("bab".match(/.(\w\w)/u), "bab".match(/.(?<a>\w\w)/u));
assert.compareArray("bab".match(/(\w\w\w)/u), "bab".match(/(?<a>\w\w\w)/u));
assert.compareArray("bab".match(/(\w\w)(\w)/u), "bab".match(/(?<a>\w\w)(?<b>\w)/u));

assert.compareArray(["bab", "b"], "bab".match(/(?<b>b).\1/u));
assert.compareArray(["baba", "b", "a"], "baba".match(/(.)(?<a>a)\1\2/u));
assert.compareArray(["baba", "b", "a", "b", "a"], "baba".match(/(.)(?<a>a)(?<b>\1)(\2)/u));
assert.compareArray(["<a", "<"], "<a".match(/(?<lt><)a/u));
assert.compareArray([">a", ">"], ">a".match(/(?<gt>>)a/u));

// Nested groups.
assert.compareArray(["bab", "bab", "ab", "b"], "bab".match(/(?<a>.(?<b>.(?<c>.)))/u));
assert.compareArray({a: "bab", b: "ab", c: "b"}, "bab".match(/(?<a>.(?<b>.(?<c>.)))/u).groups);
