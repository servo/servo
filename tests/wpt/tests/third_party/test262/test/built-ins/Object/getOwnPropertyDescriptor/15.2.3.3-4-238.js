// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-238
description: >
    Object.getOwnPropertyDescriptor - ensure that 'configurable'
    property of returned object is data property with correct
    'configurable' attribute
---*/

var obj = {
  "property": "ownDataProperty"
};

var desc = Object.getOwnPropertyDescriptor(obj, "property");

var propDefined = "configurable" in desc;

delete desc.configurable;
var propDeleted = "configurable" in desc;

assert(propDefined, 'propDefined !== true');
assert.sameValue(propDeleted, false, 'propDeleted');
