// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: PutValue operates only on references (see step 3.b).
es5id: 11.13.1-4-1
description: >
    simple assignment creates property on the global object if
    LeftHandSide is an unresolvable reference
flags: [noStrict]
---*/

  function foo() {
    __ES3_1_test_suite_test_11_13_1_unique_id_3__ = 42;
  }
  foo();

  var desc = Object.getOwnPropertyDescriptor(this, '__ES3_1_test_suite_test_11_13_1_unique_id_3__');

assert.sameValue(desc.value, 42, 'desc.value');
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
