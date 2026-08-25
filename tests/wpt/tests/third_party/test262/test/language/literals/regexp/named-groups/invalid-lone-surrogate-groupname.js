// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Lone surrogates in RegExp group names
esid: prod-GroupSpecifier
features: [regexp-named-groups]
---*/

assert.throws(SyntaxError, () => eval("/(?<a\uD801>.)/"), "Lead");
assert.throws(SyntaxError, () => eval("/(?<a\uDCA4>.)/"), "Trail");
assert.throws(SyntaxError, () => eval("/(?<a\uD801>.)/u"), "Lead with u flag");
assert.throws(SyntaxError, () => eval("/(?<a\uDCA4>.)/u"), "Trail with u flag");
