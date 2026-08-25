// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 11.1.5; 
    The production
    PropertyAssignment : set PropertyName( PropertySetParameterList ) { FunctionBody } 
    3.Let desc be the Property Descriptor{[[Set]]: closure, [[Enumerable]]: true, [[Configurable]]: true}
es5id: 11.1.5_7-3-1
description: Object literal - property descriptor for set property assignment
---*/

  var o;
  eval("o = {set foo(arg){return 1;}};");
  var desc = Object.getOwnPropertyDescriptor(o,"foo");

assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
