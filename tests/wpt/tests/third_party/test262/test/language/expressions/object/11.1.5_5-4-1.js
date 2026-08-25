// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Refer 11.1.5; 
    The production
    PropertyAssignment : PropertyName : AssignmentExpression 
    4.Let desc be the Property Descriptor{[[Value]]: propValue, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true}
es5id: 11.1.5_5-4-1
description: Object literal - property descriptor for assignment expression
---*/

  var o = {foo : 1};
  var desc = Object.getOwnPropertyDescriptor(o,"foo");

assert.sameValue(desc.value, 1, 'desc.value');
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
