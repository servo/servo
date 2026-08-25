// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.2.3-3_8
description: >
    Call arguments are evaluated before the check is made to see if
    the object is actually callable (global object)
flags: [noStrict]
---*/

    var fooCalled = false;
    function foo(){ fooCalled = true; } 
assert.throws(TypeError, function() {
        this.bar( foo() );
        throw new Test262Error("this.bar does not exist!");
});
assert.sameValue(fooCalled, true, 'fooCalled');
