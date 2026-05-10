// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: every function instance has a [[Construct]] property
es5id: 15.3.5_A3_T2
description: >
    As constructor use new Function("arg1,arg2","var x =1;
    this.y=arg1+arg2;return \"OK\";")
---*/

var FACTORY = new Function("arg1,arg2", "var x =1; this.y=arg1+arg2;return \"OK\";");
var obj = new FACTORY("1", 2);

assert.sameValue(typeof obj, "object", 'The value of `typeof obj` is expected to be "object"');
assert.sameValue(obj.constructor, FACTORY, 'The value of obj.constructor is expected to equal the value of FACTORY');
assert.sameValue(obj.y, "12", 'The value of obj.y is expected to be "12"');
