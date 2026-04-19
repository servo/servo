// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 11.1.5; 
    The production
    PropertyAssignment : get PropertyName ( ) { FunctionBody } 
    3.Let desc be the Property Descriptor{[[Get]]: closure, [[Enumerable]]: true, [[Configurable]]: true}
es5id: 11.1.5_7-3-2
description: >
    Object literal - property descriptor for set property assignment
    should not create a get function
---*/

  var o;
  eval("o = {set foo(arg){}};");
  var desc = Object.getOwnPropertyDescriptor(o,"foo");

assert.sameValue(desc.get, undefined, 'desc.get');
