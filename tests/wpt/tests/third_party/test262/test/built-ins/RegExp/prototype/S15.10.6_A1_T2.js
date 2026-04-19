// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the RegExp prototype
    object is the Object prototype
es5id: 15.10.6_A1_T2
description: >
    Add new property to Object.prototype and check it of
    RegExp.prototype
---*/

Object.prototype.indicator = 1;

assert.sameValue(RegExp.prototype.indicator, 1, 'The value of RegExp.prototype.indicator is expected to be 1');
