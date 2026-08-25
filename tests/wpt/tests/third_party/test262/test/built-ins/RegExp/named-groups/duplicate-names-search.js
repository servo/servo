// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String.prototype.search behavior with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
---*/

assert.sameValue("xab".search(/(?<x>a)|(?<x>b)/), 1);
assert.sameValue("xba".search(/(?<x>a)|(?<x>b)/), 1);
