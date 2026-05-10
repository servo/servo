// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-34
description: >
    Object.create - 'Properties' is a Date object that uses Object's
    [[Get]] method to access own enumerable property (15.2.3.7 step
    5.a)
---*/

var props = new Date(0);
props.prop = {
  value: 12,
  enumerable: true
};
var newObj = Object.create({}, props);

assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
