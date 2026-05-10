// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.3_b
description: >
    Tests that Intl.Collator.prototype functions throw a  TypeError if
    called on a non-object value or an object that hasn't been
    initialized as a Collator.
author: Norbert Lindenberg
---*/

var functions = {
    "compare getter": Object.getOwnPropertyDescriptor(Intl.Collator.prototype, "compare").get,
    resolvedOptions: Intl.Collator.prototype.resolvedOptions
};
var invalidTargets = [undefined, null, true, 0, "Collator", [], {}];

Object.getOwnPropertyNames(functions).forEach(function (functionName) {
    var f = functions[functionName];
    invalidTargets.forEach(function (target) {
        assert.throws(TypeError, function() {
            f.call(target);
        }, "Calling " + functionName + " on " + target + " was not rejected.");
    });
});
