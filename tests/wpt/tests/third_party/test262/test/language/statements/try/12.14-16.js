// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.14-16
description: >
    Exception object is a function which update in catch block, when
    an exception parameter is called as a function in catch block,
    global object is passed as the this value
flags: [noStrict]
---*/

var global = this;
var result;

(function() {
        try {
            throw function () {
                this._12_14_16_foo = "test";
            };
        } catch (e) {
            var obj = {};
            obj.test = function () {
                this._12_14_16_foo = "test1";
            };
            e = obj.test;
            e();
            result = global._12_14_16_foo;
        }
})();

assert.sameValue(result, "test1", 'result');
