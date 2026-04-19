// Copyright 2022 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Matching behavior with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
includes: [compareArray.js]
---*/

assert.compareArray("bab".match(/(?<x>a)|(?<x>b)/), ["b", undefined, "b"]);
assert.compareArray("bab".match(/(?<x>b)|(?<x>a)/), ["b", "b", undefined]);

assert.compareArray("aa".match(/(?:(?<x>a)|(?<x>b))\k<x>/), ["aa", "a", undefined]);
assert.compareArray("bb".match(/(?:(?<x>a)|(?<x>b))\k<x>/), ["bb", undefined, "b"]);

let matchResult = "aabb".match(/(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/);
assert.compareArray(matchResult, ["aabb", undefined, "b"]);
assert.sameValue(matchResult.groups.x, "b");

assert.sameValue("abab".match(/(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/), null);

assert.sameValue("abab".match(/(?:(?<x>a)|(?<x>b))\k<x>/), null);

assert.sameValue("cdef".match(/(?:(?<x>a)|(?<x>b))\k<x>/), null);

assert.compareArray("xx".match(/^(?:(?<a>x)|(?<a>y)|z)\k<a>$/), ["xx", "x", undefined]);
assert.compareArray("z".match(/^(?:(?<a>x)|(?<a>y)|z)\k<a>$/), ["z", undefined, undefined]);
assert.sameValue("zz".match(/^(?:(?<a>x)|(?<a>y)|z)\k<a>$/), null);
assert.compareArray("zy".match(/(?<a>x)|(?:zy\k<a>)/), ["zy", undefined]);

assert.compareArray("xz".match(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/), ["xz", undefined, undefined]);
assert.compareArray("yz".match(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/), ["yz", undefined, undefined]);
assert.sameValue("xzx".match(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/), null);
assert.sameValue("yzy".match(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/), null);
