// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String.prototype.replaceAll behavior with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
---*/

assert.sameValue("abxab".replaceAll(/(?<x>a)|(?<x>b)/g, "[$<x>]"), "[a][b]x[a][b]");
assert.sameValue("baxba".replaceAll(/(?<x>a)|(?<x>b)/g, "[$<x>]"), "[b][a]x[b][a]");

assert.sameValue("abxab".replaceAll(/(?<x>a)|(?<x>b)/g, "[$<x>][$1][$2]"), "[a][a][][b][][b]x[a][a][][b][][b]");
assert.sameValue("baxba".replaceAll(/(?<x>a)|(?<x>b)/g, "[$<x>][$1][$2]"), "[b][][b][a][a][]x[b][][b][a][a][]");

