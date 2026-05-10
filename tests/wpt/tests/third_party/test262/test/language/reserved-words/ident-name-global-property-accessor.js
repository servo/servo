// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6.1-4-16
description: >
    Allow global constant properties as property names by accessor function within an object.
---*/

var test;

var tokenCodes = {
    set undefined(value) { test = 'undefined'; },
    get undefined() { return 'undefined'; },
    set NaN(value) { test = 'NaN'; },
    get NaN() { return 'NaN'; },
    set Infinity(value) { test = 'Infinity'; },
    get Infinity() { return 'Infinity'; },
};

var arr = [
    'undefined',
    'NaN',
    'Infinity',
];

for (var i = 0; i < arr.length; ++i) {
    var propertyName = arr[i];

    assert(tokenCodes.hasOwnProperty(propertyName),
           'Property "' + propertyName + '" found');

    assert.sameValue(tokenCodes[propertyName], propertyName,
                     'Property "' + propertyName + '" has correct value');

    tokenCodes[propertyName] = 0;
    assert.sameValue(test, propertyName,
                     'Property "' + propertyName + '" sets correct value');
}
