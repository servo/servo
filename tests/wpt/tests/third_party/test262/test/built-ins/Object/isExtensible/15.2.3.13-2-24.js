// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.13-2-24
description: >
    Object.isExtensible returns true if O is extensible and has a
    prototype that is extensible
---*/

var proto = {};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var obj = new ConstructFun();

assert(Object.isExtensible(obj), 'Object.isExtensible(obj) !== true');
