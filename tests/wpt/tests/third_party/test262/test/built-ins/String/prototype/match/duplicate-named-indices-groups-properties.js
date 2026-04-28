// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Properties on indices.groups object with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups, regexp-match-indices]
includes: [compareArray.js]
---*/

const matcher = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/d;

const threeMatchResult = "abc".match(matcher);
assert.compareArray(threeMatchResult.indices.groups.x, [1, 2], "group x matches b");
assert.compareArray(threeMatchResult.indices.groups.y, [0, 1], "group y matches a");
assert.compareArray(threeMatchResult.indices.groups.z, [2, 3], "group z matches c");
assert.compareArray(
  Object.keys(threeMatchResult.indices.groups),
  ["x", "y", "z"],
  "Properties of groups are ordered in RegExp source order despite y matching before x in this alternative"
);

const twoMatchResult = "ad".match(matcher);
assert.compareArray(twoMatchResult.indices.groups.x, [0, 1], "group x matches a");
assert.sameValue(twoMatchResult.indices.groups.y, undefined, "group y does not match");
assert.compareArray(twoMatchResult.indices.groups.z, [1, 2], "group z matches d");
assert.compareArray(
  Object.keys(twoMatchResult.indices.groups),
  ["x", "y", "z"],
  "y is still present on groups object, in the right order, despite not matching"
);

const iteratedMatcher = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/d;

const matchedInPrevIterationResult = "aac".match(iteratedMatcher);
assert.sameValue(matchedInPrevIterationResult.indices.groups.x, undefined, "group x does not match in the last iteration");
