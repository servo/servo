// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Matching behavior with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
---*/

assert(/(?<x>a)|(?<x>b)/.test("bab"));
assert(/(?<x>b)|(?<x>a)/.test("bab"));

assert(/(?:(?<x>a)|(?<x>b))\k<x>/.test("aa"));
assert(/(?:(?<x>a)|(?<x>b))\k<x>/.test("bb"));

let matchResult = /(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/.test("aabb");
assert(matchResult);

let notMatched = /(?:(?:(?<x>a)|(?<x>b))\k<x>){2}/.test("abab");
assert.sameValue(notMatched, false);
