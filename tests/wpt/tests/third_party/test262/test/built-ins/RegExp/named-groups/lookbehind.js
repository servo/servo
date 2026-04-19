// Copyright 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Named groups can be used in conjunction with lookbehind
esid: prod-GroupSpecifier
features: [regexp-named-groups, regexp-lookbehind]
includes: [compareArray.js]
---*/

// Unicode mode
assert.compareArray(["f", "c"], "abcdef".match(/(?<=(?<a>\w){3})f/u));
assert.sameValue("c", "abcdef".match(/(?<=(?<a>\w){3})f/u).groups.a);
assert.sameValue("b", "abcdef".match(/(?<=(?<a>\w){4})f/u).groups.a);
assert.sameValue("a", "abcdef".match(/(?<=(?<a>\w)+)f/u).groups.a);
assert.sameValue(null, "abcdef".match(/(?<=(?<a>\w){6})f/u));

assert.compareArray(["f", ""], "abcdef".match(/((?<=\w{3}))f/u));
assert.compareArray(["f", ""], "abcdef".match(/(?<a>(?<=\w{3}))f/u));

assert.compareArray(["f", undefined], "abcdef".match(/(?<!(?<a>\d){3})f/u));
assert.sameValue(null, "abcdef".match(/(?<!(?<a>\D){3})f/u));

assert.compareArray(["f", undefined], "abcdef".match(/(?<!(?<a>\D){3})f|f/u));
assert.compareArray(["f", undefined], "abcdef".match(/(?<a>(?<!\D{3}))f|f/u));

// Non-Unicode mode
assert.compareArray(["f", "c"], "abcdef".match(/(?<=(?<a>\w){3})f/));
assert.sameValue("c", "abcdef".match(/(?<=(?<a>\w){3})f/).groups.a);
assert.sameValue("b", "abcdef".match(/(?<=(?<a>\w){4})f/).groups.a);
assert.sameValue("a", "abcdef".match(/(?<=(?<a>\w)+)f/).groups.a);
assert.sameValue(null, "abcdef".match(/(?<=(?<a>\w){6})f/));

assert.compareArray(["f", ""], "abcdef".match(/((?<=\w{3}))f/));
assert.compareArray(["f", ""], "abcdef".match(/(?<a>(?<=\w{3}))f/));

assert.compareArray(["f", undefined], "abcdef".match(/(?<!(?<a>\d){3})f/));
assert.sameValue(null, "abcdef".match(/(?<!(?<a>\D){3})f/));

assert.compareArray(["f", undefined], "abcdef".match(/(?<!(?<a>\D){3})f|f/));
assert.compareArray(["f", undefined], "abcdef".match(/(?<a>(?<!\D{3}))f|f/));

// Even within a lookbehind, properties are created in left to right order
assert.compareArray(["fst", "snd"], Object.getOwnPropertyNames(
    /(?<=(?<fst>.)|(?<snd>.))/u.exec("abcd").groups));
