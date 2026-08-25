// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator returns false when deleting a direct reference to
    a function argument
flags: [noStrict]
---*/

function foo(a, b) {
  // Now, deleting 'a' directly should fail
  // because 'a' is direct reference to a function argument;
  var d = delete a;
  return d === false && a === 1;
}

assert(foo(1, 2), 'foo(1,2) !== true');
