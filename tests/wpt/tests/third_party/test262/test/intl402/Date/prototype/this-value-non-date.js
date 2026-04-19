// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.3.0_1
description: >
    Tests that Date.prototype.toLocaleString & Co. handle "this time
    value" correctly.
author: Norbert Lindenberg
---*/

var functions = {
    toLocaleString: Date.prototype.toLocaleString,
    toLocaleDateString: Date.prototype.toLocaleDateString,
    toLocaleTimeString: Date.prototype.toLocaleTimeString
};
var invalidValues = [undefined, null, 5, "5", false, {valueOf: function () { return 5; }}];

Object.getOwnPropertyNames(functions).forEach(function (p) {
    var f = functions[p];
    invalidValues.forEach(function (value) {
        assert.throws(TypeError, function() {
            var result = f.call(value);
        }, "Date.prototype." + p + " did not reject this = " + value + ".");
    });
});
