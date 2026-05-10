// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Named backreferences in non-Unicode RegExps
esid: prod-GroupSpecifier
features: [regexp-named-groups]
includes: [compareArray.js]
---*/

// Named references.
assert.compareArray(["bab", "b"], "bab".match(/(?<b>.).\k<b>/));
assert.sameValue(null, "baa".match(/(?<b>.).\k<b>/));

// Reference inside group.
assert.compareArray(["bab", "b"], "bab".match(/(?<a>\k<a>\w)../));
assert.sameValue("b", "bab".match(/(?<a>\k<a>\w)../).groups.a);

// Reference before group.
assert.compareArray(["bab", "b"], "bab".match(/\k<a>(?<a>b)\w\k<a>/));
assert.sameValue("b", "bab".match(/\k<a>(?<a>b)\w\k<a>/).groups.a);
assert.compareArray(["bab", "b", "a"], "bab".match(/(?<b>b)\k<a>(?<a>a)\k<b>/));
let {a, b} = "bab".match(/(?<b>b)\k<a>(?<a>a)\k<b>/).groups;
assert.sameValue(a, "a");
assert.sameValue(b, "b");

assert.compareArray(["bab", "b"], "bab".match(/\k<a>(?<a>b)\w\k<a>/));
assert.compareArray(["bab", "b", "a"], "bab".match(/(?<b>b)\k<a>(?<a>a)\k<b>/));

// Reference properties.
assert.sameValue("a", /(?<a>a)(?<b>b)\k<a>/.exec("aba").groups.a);
assert.sameValue("b", /(?<a>a)(?<b>b)\k<a>/.exec("aba").groups.b);
assert.sameValue(undefined, /(?<a>a)(?<b>b)\k<a>/.exec("aba").groups.c);
assert.sameValue(undefined, /(?<a>a)(?<b>b)\k<a>|(?<c>c)/.exec("aba").groups.c);
