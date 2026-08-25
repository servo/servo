// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Properties of the groups object of indices are created with CreateDataProperty
includes: [compareArray.js, propertyHelper.js]
esid: sec-makeindicesarray
features: [regexp-named-groups, regexp-match-indices]
info: |
  MakeIndicesArray ( S, indices, groupNames, hasIndices )
    13. For each integer _i_ such that _i_ >= 0 and _i_ < _n_, do
      e. If _i_ > 0 and _groupNames_[_i_ - 1] is not *undefined*, then
        i. Perform ! CreateDataProperty(_groups_, _groupNames_[_i_ - 1], _matchIndicesArray_).
---*/


// Properties created on result.groups in textual order.
let groupNames = Object.getOwnPropertyNames(/(?<fst>.)|(?<snd>.)/du.exec("abcd").indices.groups);
assert.compareArray(groupNames, ["fst", "snd"]);

// Properties are created with Define, not Set
let counter = 0;
Object.defineProperty(Object.prototype, 'x', {set() { counter++; }});

let indices = /(?<x>.)/d.exec('a').indices;
let groups = indices.groups;
assert.sameValue(counter, 0);

// Properties are writable, enumerable and configurable
// (from CreateDataProperty)
verifyProperty(groups, 'x', {
    writable: true,
    enumerable: true,
    configurable: true
});
