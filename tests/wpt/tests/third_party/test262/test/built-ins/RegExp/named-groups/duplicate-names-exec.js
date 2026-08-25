// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Matching behavior with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
includes: [compareArray.js]
---*/

assert.compareArray(/(?<x>a)|(?<x>b)/.exec("bab"), ["b", undefined, "b"]);
assert.compareArray(/(?<x>b)|(?<x>a)/.exec("bab"), ["b", "b", undefined]);

assert.compareArray(/(?:(?<x>a)|(?<x>b))\k<x>/.exec("aa"), ["aa", "a", undefined]);
assert.compareArray(/(?:(?<x>a)|(?<x>b))\k<x>/.exec("bb"), ["bb", undefined, "b"]);

let matchResult = /(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/.exec("aabb");
assert.compareArray(matchResult, ["aabb", undefined, "b"]);
assert.sameValue(matchResult.groups.x, "b");

assert.sameValue(/(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/.exec("abab"), null);

assert.sameValue(/(?:(?<x>a)|(?<x>b))\k<x>/.exec("abab"), null);

assert.sameValue(/(?:(?<x>a)|(?<x>b))\k<x>/.exec("cdef"), null);

assert.compareArray(/^(?:(?<a>x)|(?<a>y)|z)\k<a>$/.exec("xx"), ["xx", "x", undefined]);
assert.compareArray(/^(?:(?<a>x)|(?<a>y)|z)\k<a>$/.exec("z"), ["z", undefined, undefined]);
assert.sameValue(/^(?:(?<a>x)|(?<a>y)|z)\k<a>$/.exec("zz"), null);
assert.compareArray(/(?<a>x)|(?:zy\k<a>)/.exec("zy"), ["zy", undefined]);

assert.compareArray(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/.exec("xz"), ["xz", undefined, undefined]);
assert.compareArray(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/.exec("yz"), ["yz", undefined, undefined]);
assert.sameValue(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/.exec("xzx"), null);
assert.sameValue(/^(?:(?<a>x)|(?<a>y)|z){2}\k<a>$/.exec("yzy"), null);
