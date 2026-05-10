// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.14-15
description: >
    Exception object is a function which is a property of an object,
    when an exception parameter is called as a function in catch
    block, global object is passed as the this value
flags: [noStrict]
---*/

var global = this;
var result;

(function() {
        var obj = {};
        obj.test = function () {
            this._12_14_15_foo = "test";
        };
        try {
            throw obj.test;
        } catch (e) {
            e();
            result = global._12_14_15_foo;
        }
})();

assert.sameValue(result, "test", 'result');
