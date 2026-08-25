// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-227
description: >
    Object.getOwnPropertyDescriptor - ensure that 'writable' property
    of returned object is data property with correct 'value' attribute
---*/

var obj = {
  "property": "ownDataProperty"
};

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert.sameValue(desc.writable, true, 'desc.writable');
