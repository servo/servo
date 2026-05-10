// Copyright 2022 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: match indices with duplicate named capture groups
esid: sec-makematchindicesindexpairarray
features: [regexp-duplicate-named-groups, regexp-match-indices]
includes: [compareArray.js]
---*/

let indices = "..ab".match(/(?<x>a)|(?<x>b)/d).indices;
assert.compareArray(indices.groups.x, [2, 3]);

indices = "..ba".match(/(?<x>a)|(?<x>b)/d).indices;
assert.compareArray(indices.groups.x, [2, 3]);
