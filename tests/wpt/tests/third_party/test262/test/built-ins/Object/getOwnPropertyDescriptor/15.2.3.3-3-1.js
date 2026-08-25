// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-1
description: Object.getOwnPropertyDescriptor - 'P' is own data property
---*/

var obj = {
  property: "ownDataProperty"
};

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert.sameValue(desc.value, "ownDataProperty", 'desc.value');
