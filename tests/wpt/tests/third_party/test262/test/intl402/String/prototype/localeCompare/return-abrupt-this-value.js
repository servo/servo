// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.1.1_1
description: >
    Tests that localeCompare rejects values that can't be coerced to
    an object.
author: Norbert Lindenberg
---*/

var invalidValues = [undefined, null];
 
invalidValues.forEach(function (value) {
    assert.throws(TypeError, function() {
        var result = String.prototype.localeCompare.call(value, "");
    }, "String.prototype.localeCompare did not reject this = " + value + ".");
});
