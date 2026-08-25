// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-2
description: Object.getOwnPropertyDescriptor - 'P' is inherited data property
---*/

var proto = {
  property: "inheritedDataProperty"
};

var Con = function() {};
Con.ptototype = proto;

var child = new Con();

var desc = Object.getOwnPropertyDescriptor(child, "property");

assert.sameValue(typeof desc, "undefined", 'typeof desc');
