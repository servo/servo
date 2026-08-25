// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Properties on groups object with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
includes: [compareArray.js]
---*/

const matcher = /(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))/;

const threeMatchResult = "abc".match(matcher);
assert.sameValue(threeMatchResult.groups.x, "b", "group x matches b");
assert.sameValue(threeMatchResult.groups.y, "a", "group y matches a");
assert.sameValue(threeMatchResult.groups.z, "c", "group z matches c");
assert.compareArray(
  Object.keys(threeMatchResult.groups),
  ["x", "y", "z"],
  "Properties of groups are ordered in RegExp source order despite y matching before x in this alternative"
);

const twoMatchResult = "ad".match(matcher);
assert.sameValue(twoMatchResult.groups.x, "a", "group x matches a");
assert.sameValue(twoMatchResult.groups.y, undefined, "group y does not match");
assert.sameValue(twoMatchResult.groups.z, "d", "group z matches d");
assert.compareArray(
  Object.keys(twoMatchResult.groups),
  ["x", "y", "z"],
  "y is still present on groups object, in the right order, despite not matching"
);

const iteratedMatcher = /(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}/;

const matchedInPrevIterationResult = "aac".match(iteratedMatcher);
assert.sameValue(matchedInPrevIterationResult.groups.x, undefined, "group x does not match in the last iteration");
