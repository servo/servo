// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-GroupSpecifier
description: >
  \k is parsed as IdentityEscape as look-behind assertion is not a GroupName.
features: [regexp-named-groups, regexp-lookbehind]
---*/

assert(/\k<a>(?<=>)a/.test("k<a>a"));
assert(/(?<=>)\k<a>/.test(">k<a>"));

assert(/\k<a>(?<!a)a/.test("k<a>a"));
assert(/(?<!a>)\k<a>/.test("k<a>"));
