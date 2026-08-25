// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.5-5-b_1
description: >
    Changing the value of an accessor property should not affect it's
    property descriptor attributes.
---*/

    var tempObj = {};
    
    Object.defineProperty(tempObj, "reduce", { get: function() {return 456;}, enumerable:false, set: function() {;}});
    var origDesc = Object.getOwnPropertyDescriptor(tempObj, "reduce");

    var newDesc;

        tempObj.reduce = 123;
        newDesc = Object.getOwnPropertyDescriptor(tempObj, "reduce");
        var descArray = [origDesc, newDesc];
        
        for (var j in descArray) {
            for (var i in descArray[j]) {
                assert.sameValue(origDesc[i], newDesc[i], 'origDesc[i]');
            }
        }

assert.sameValue(tempObj.reduce, 456, 'tempObj.reduce');
