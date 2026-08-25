// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.2.3-3_6
description: >
    Call arguments are evaluated before the check is made to see if
    the object is actually callable (getter called)
---*/

    var o = { }; 
    Object.defineProperty(o, "bar", {get: function()  {this.barGetter = true; return 42;}, 
                                     set: function(x) {this.barSetter = true; }});
assert.throws(TypeError, function() {
        o.foo( o.bar );
        throw new Test262Error("o.foo does not exist!");
});
assert.sameValue(o.barGetter, true, 'o.barGetter');
assert.sameValue(o.barSetter, undefined, 'o.barSetter');
