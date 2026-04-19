// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Named groups in Unicode RegExps have some syntax errors and some
  compatibility escape fallback behavior.
esid: prod-GroupSpecifier
features: [regexp-named-groups]
includes: [compareArray.js]
---*/

assert(/\k<a>/.test("k<a>"));
assert(/\k<4>/.test("k<4>"));
assert(/\k<a/.test("k<a"));
assert(/\k/.test("k"));

assert(/(?<a>\a)/.test("a"));

assert.compareArray(["k<a>"], "xxxk<a>xxx".match(/\k<a>/));
assert.compareArray(["k<a"], "xxxk<a>xxx".match(/\k<a/));

assert(/\k<a>(<a>x)/.test("k<a><a>x"));
assert(/\k<a>\1/.test("k<a>\x01"));
assert(/\1(b)\k<a>/.test("bk<a>"));
