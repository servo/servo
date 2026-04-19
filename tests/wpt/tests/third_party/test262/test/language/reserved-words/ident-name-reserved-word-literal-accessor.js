// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6.1-4-1
description: >
    Allow reserved words as property names by accessor function within an object.
---*/

var test;

var tokenCodes = {
    set null(value) { test = 'null'; },
    get null() { return 'null'; },
    set true(value) { test = 'true'; },
    get true() { return 'true'; },
    set false(value) { test = 'false'; },
    get false() { return 'false'; },
};

var arr = [
        'null',
        'true',
        'false',
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
