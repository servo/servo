// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.14.4-8-b_2
description: Non-writable property on a prototype written to in strict mode.
flags: [onlyStrict]
---*/

    function foo() {};
    Object.defineProperty(foo.prototype, "bar", {value: "unwritable"}); 
    
    var o = new foo(); 
assert.throws(TypeError, function() {
        o.bar = "overridden";
});
assert.sameValue(o.bar, "unwritable", 'o.bar');
