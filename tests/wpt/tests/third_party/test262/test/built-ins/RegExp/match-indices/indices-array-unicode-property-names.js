// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Basic matching cases with non-unicode matches.
includes: [compareArray.js]
esid: sec-makeindicesarray
features: [regexp-named-groups, regexp-match-indices]
---*/

assert.compareArray([1, 2], /(?<π>a)/du.exec("bab").indices.groups.π);
assert.compareArray([1, 2], /(?<\u{03C0}>a)/du.exec("bab").indices.groups.π);
assert.compareArray([1, 2], /(?<π>a)/du.exec("bab").indices.groups.\u03C0);
assert.compareArray([1, 2], /(?<\u{03C0}>a)/du.exec("bab").indices.groups.\u03C0);
assert.compareArray([1, 2], /(?<$>a)/du.exec("bab").indices.groups.$);
assert.compareArray([1, 2], /(?<_>a)/du.exec("bab").indices.groups._);
assert.compareArray([1, 2], /(?<$𐒤>a)/du.exec("bab").indices.groups.$𐒤);
assert.compareArray([1, 2], /(?<_\u200C>a)/du.exec("bab").indices.groups._\u200C);
assert.compareArray([1, 2], /(?<_\u200D>a)/du.exec("bab").indices.groups._\u200D);
assert.compareArray([1, 2], /(?<ಠ_ಠ>a)/du.exec("bab").indices.groups.ಠ_ಠ);
