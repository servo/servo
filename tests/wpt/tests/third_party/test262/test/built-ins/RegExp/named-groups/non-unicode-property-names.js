// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Exotic named group names in non-Unicode RegExps
esid: prod-GroupSpecifier
features: [regexp-named-groups]
---*/

assert.sameValue("a", /(?<π>a)/.exec("bab").groups.π);
assert.sameValue("a", /(?<π>a)/.exec("bab").groups.\u03C0);
assert.sameValue("a", /(?<$>a)/.exec("bab").groups.$);
assert.sameValue("a", /(?<_>a)/.exec("bab").groups._);
assert.sameValue("a", /(?<_\u200C>a)/.exec("bab").groups._\u200C);
assert.sameValue("a", /(?<_\u200D>a)/.exec("bab").groups._\u200D);
assert.sameValue("a", /(?<ಠ_ಠ>a)/.exec("bab").groups.ಠ_ಠ);

// Unicode escapes in capture names.
assert(/(?<\u0041>.)/.test("a"));
assert(RegExp("(?<\u{0041}>.)").test("a"), "Non-surrogate");

// 4-char escapes must be the proper ID_Start/ID_Continue
assert(RegExp("(?<\\u0041>.)").test("a"), "Non-surrogate");
