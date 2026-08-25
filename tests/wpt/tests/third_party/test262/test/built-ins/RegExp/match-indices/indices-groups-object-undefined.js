// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The groups object of indices is created unconditionally.
includes: [propertyHelper.js]
esid: sec-makeindicesarray
features: [regexp-named-groups, regexp-match-indices]
info: |
  MakeIndicesArray ( S, indices, groupNames, hasGroups )
    10. If _hasGroups_ is *true*, then
      a. Let _groups_ be ! ObjectCreate(*null*).
    11. Else,
      a. Let _groups_ be *undefined*.
    12. Perform ! CreateDataProperty(_A_, `"groups"`, _groups_).
---*/


const re = /./d;
const indices = re.exec("a").indices;
verifyProperty(indices, 'groups', {
  writable: true,
  enumerable: true,
  configurable: true,
  value: undefined
});
