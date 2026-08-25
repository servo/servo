// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6.1-2-16
description: >
    Allow global constant properties as property names by dot operator assignment
---*/

var tokenCodes = {};

tokenCodes.undefined = 'undefined';
tokenCodes.NaN = 'NaN';
tokenCodes.Infinity = 'Infinity';

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
}
