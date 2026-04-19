// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.3_b
description: >
    Tests that Intl.DateTimeFormat.prototype functions throw a
    TypeError if called on a non-object value or an object that hasn't
    been  initialized as a DateTimeFormat.
author: Norbert Lindenberg
---*/

var functions = {
    "format getter": Object.getOwnPropertyDescriptor(Intl.DateTimeFormat.prototype, "format").get,
    resolvedOptions: Intl.DateTimeFormat.prototype.resolvedOptions
};
var invalidTargets = [undefined, null, true, 0, "DateTimeFormat", [], {}];

Object.getOwnPropertyNames(functions).forEach(function (functionName) {
    var f = functions[functionName];
    invalidTargets.forEach(function (target) {
        assert.throws(TypeError, function() {
            f.call(target);
        }, "Calling " + functionName + " on " + target + " was not rejected.");
    });
});
