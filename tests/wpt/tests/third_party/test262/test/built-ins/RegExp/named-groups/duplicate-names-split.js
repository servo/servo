// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String.prototype.search behavior with duplicate named capture groups
esid: prod-GroupSpecifier
includes: [compareArray.js]
features: [regexp-duplicate-named-groups]
---*/

assert.compareArray("xab".split(/(?<x>a)|(?<x>b)/), ["x", "a", undefined, "", undefined, "b", ""]);
assert.compareArray("xba".split(/(?<x>a)|(?<x>b)/), ["x", undefined, "b", "", "a", undefined, ""]);
