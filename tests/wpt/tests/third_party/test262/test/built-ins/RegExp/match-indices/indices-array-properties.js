// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The properties of the "indices" array are created with CreateDataProperty.
includes: [propertyHelper.js]
esid: sec-makeindicesarray
features: [regexp-match-indices]
info: |
  MakeIndicesArray ( S, indices, groupNames, hasGroups )
    13. For each integer _i_ such that _i_ >= 0 and _i_ < _n_, do
      d. Perform ! CreateDataProperty(_A_, ! ToString(_n_), _matchIndicesArray_).
---*/

let input = "abcd";
let match = /b(c)/d.exec(input);
let indices = match.indices;

verifyProperty(indices, '0', {
  enumerable: true,
  configurable: true,
  writable: true
});

verifyProperty(indices, '1', {
  enumerable: true,
  configurable: true,
  writable: true
});
