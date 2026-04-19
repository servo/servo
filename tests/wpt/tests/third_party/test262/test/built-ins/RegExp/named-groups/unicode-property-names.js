// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Exotic named group names in Unicode RegExps
esid: prod-GroupSpecifier
features: [regexp-named-groups]
---*/

assert.sameValue("a", /(?<π>a)/u.exec("bab").groups.π);
assert.sameValue("a", /(?<\u{03C0}>a)/u.exec("bab").groups.π);
assert.sameValue("a", /(?<π>a)/u.exec("bab").groups.\u03C0);
assert.sameValue("a", /(?<\u{03C0}>a)/u.exec("bab").groups.\u03C0);
assert.sameValue("a", /(?<$>a)/u.exec("bab").groups.$);
assert.sameValue("a", /(?<_>a)/u.exec("bab").groups._);
assert.sameValue("a", /(?<$𐒤>a)/u.exec("bab").groups.$𐒤);
assert.sameValue("a", /(?<_\u200C>a)/u.exec("bab").groups._\u200C);
assert.sameValue("a", /(?<_\u200D>a)/u.exec("bab").groups._\u200D);
assert.sameValue("a", /(?<ಠ_ಠ>a)/u.exec("bab").groups.ಠ_ಠ);

// Unicode escapes in capture names.
assert(/(?<a\uD801\uDCA4>.)/u.test("a"), "\\u Lead \\u Trail");
assert(/(?<\u0041>.)/u.test("a"), "\\u NonSurrogate");
assert(/(?<\u{0041}>.)/u.test("a"), "\\u{ Non-surrogate }");
assert(/(?<a\u{104A4}>.)/u.test("a"), "\\u{ Surrogate, ID_Continue }");
assert(RegExp("(?<\u{0041}>.)", "u").test("a"), "Non-surrogate");
assert(RegExp("(?<a\u{104A4}>.)", "u").test("a"), "Surrogate,ID_Continue");
assert((/(?<\u{0041}>.)/u).test("a"), "Non-surrogate");
assert(/(?<a\u{104A4}>.)/u.test("a"), "Surrogate, ID_Continue");
assert(RegExp("(?<\\u0041>.)", "u").test("a"), "Non-surrogate");
