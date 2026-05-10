// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.3.2_FDT_1
description: Tests that format handles non-finite values correctly.
author: Norbert Lindenberg
---*/

var invalidValues = [NaN, Infinity, -Infinity];

var format = new Intl.DateTimeFormat();

invalidValues.forEach(function (value) {
    assert.throws(RangeError, function() {
        var result = format.format(value);
    }, "Invalid value " + value + " was not rejected.");
});
