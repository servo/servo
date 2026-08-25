// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.5-3-b_2
description: >
    Changing the value of a data property should not affect it's
    non-value property descriptor attributes.
---*/

    var tempObj = {};
    
    Object.defineProperty(tempObj, "reduce", { value:456, enumerable:false, writable:true});
    var origDesc = Object.getOwnPropertyDescriptor(tempObj, "reduce");

    var newDesc;

        tempObj.reduce = 123;
        newDesc = Object.getOwnPropertyDescriptor(tempObj, "reduce");
        var descArray = [origDesc, newDesc];
        
        for (var j in descArray) {
            for (var i in descArray[j]) {
                if (i==="value") {
                    assert.notSameValue(origDesc[i], newDesc[i], 'origDesc[i]');
                }
                else {
                    assert.sameValue(origDesc[i], newDesc[i], 'origDesc[i]');
                }
            }
        }
