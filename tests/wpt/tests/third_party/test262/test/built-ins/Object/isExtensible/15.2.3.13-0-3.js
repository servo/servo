// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A newly created object using the Object contructor has its [[Extensible]]
    property set to true by default (15.2.2.1, step 8).
es5id: 15.2.3.13-0-3
description: >
    Object.isExtensible is true for objects created using the Object
    constructor
---*/

var o = new Object();

assert.sameValue(Object.isExtensible(o), true, 'Object.isExtensible(o)');
