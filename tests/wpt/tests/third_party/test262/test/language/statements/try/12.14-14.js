// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.14-14
description: >
    Exception object is a function, when an exception parameter is
    called as a function in catch block, global object is passed as
    the this value
flags: [noStrict]
---*/

var global = this;
var result;

(function() {
        try {
            throw function () {
                this._12_14_14_foo = "test";
            };
        } catch (e) {
            e();
            result = global._12_14_14_foo;
        }
})();

assert.sameValue(result, "test", 'result');
