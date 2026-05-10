// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-3-24
description: >
    Object.preventExtensions - [[Extensible]]: false on a prototype
    doesn't prevent adding properties to an instance that inherits
    from that prototype
---*/

var proto = {};
var preCheck = Object.isExtensible(proto);
Object.preventExtensions(proto);

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var child = new ConstructFun();

child.prop = 10;

assert(preCheck, 'preCheck !== true');
assert(child.hasOwnProperty("prop"), 'child.hasOwnProperty("prop") !== true');
