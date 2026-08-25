// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-39
description: >
    Object.create - ensure that side-effects of gets occur in the same
    order as they would for: for (P in props) props[P] (15.2.3.7 step
    5.a)
---*/

var props = {};
props.prop1 = {
  value: 12,
  enumerable: true
};
props.prop2 = {
  value: true,
  enumerable: true
};

var tempArray = [];
for (var p in props) {
  if (props.hasOwnProperty(p)) {
    tempArray.push(p);
  }
}

var newObj = Object.create({}, props);
var index = 0;
for (var q in newObj) {
  assert.sameValue(tempArray[index++] !== q && newObj.hasOwnProperty(q), false, 'tempArray[index++] !== q && newObj.hasOwnProperty(q)');
}
