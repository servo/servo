// Copyright 2022 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Enumeration order of the groups object with duplicate named capture groups
esid: prod-GroupSpecifier
features: [regexp-duplicate-named-groups]
includes: [compareArray.js]
---*/


let regexp = /(?<y>a)(?<x>a)|(?<x>b)(?<y>b)/;

assert.compareArray(
  Object.keys(regexp.exec("aa").groups),
  ["y", "x"],
  "property enumeration order of the groups object is based on source order, not match order"
);

assert.compareArray(
  Object.keys(regexp.exec("bb").groups),
  ["y", "x"],
  "property enumeration order of the groups object is based on source order, not match order"
);
