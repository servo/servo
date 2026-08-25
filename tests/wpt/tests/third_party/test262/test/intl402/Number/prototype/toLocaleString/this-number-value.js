// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2.1_1
description: Tests that toLocaleString handles "this Number value" correctly.
author: Norbert Lindenberg
---*/

var invalidValues = [undefined, null, "5", false, {valueOf: function () { return 5; }}];
var validValues = [5, NaN, -1234567.89, -Infinity];

invalidValues.forEach(function (value) {
    assert.throws(TypeError, function() {
        var result = Number.prototype.toLocaleString.call(value);
    }, "Number.prototype.toLocaleString did not reject this = " + value + ".");
});

// for valid values, just check that a Number value and the corresponding
// Number object get the same result.
validValues.forEach(function (value) {
    var Constructor = Number; // to keep jshint happy
    var valueResult = Number.prototype.toLocaleString.call(value);
    var objectResult = Number.prototype.toLocaleString.call(new Constructor(value));
    assert.sameValue(valueResult, objectResult, "Number.prototype.toLocaleString produces different results for Number value " + value + " and corresponding Number object.");
});
