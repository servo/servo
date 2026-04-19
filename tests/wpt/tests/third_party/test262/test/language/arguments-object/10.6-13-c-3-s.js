// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-13-c-3-s
description: arguments.callee is non-configurable in strict mode
flags: [onlyStrict]
---*/

function testcase() {
  var desc = Object.getOwnPropertyDescriptor(arguments,"callee");

  assert.sameValue(desc.configurable, false, 'desc.configurable');
  assert.sameValue(desc.enumerable, false, 'desc.enumerable');
  assert.sameValue(desc.hasOwnProperty('value'), false, 'desc.hasOwnProperty("value")');
  assert.sameValue(desc.hasOwnProperty('writable'), false, 'desc.hasOwnProperty("writable")');
  assert.sameValue(desc.hasOwnProperty('get'), true, 'desc.hasOwnProperty("get")');
  assert.sameValue(desc.hasOwnProperty('set'), true, 'desc.hasOwnProperty("set")');
}
testcase();
