// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-3
description: >
    Object.getOwnPropertyDescriptor - 'P' is own data property that
    overrides an inherited data property
---*/

var proto = {
  property: "inheritedDataProperty"
};

var Con = function() {};
Con.ptototype = proto;

var child = new Con();
child.property = "ownDataProperty";

var desc = Object.getOwnPropertyDescriptor(child, "property");

assert.sameValue(desc.value, "ownDataProperty", 'desc.value');
