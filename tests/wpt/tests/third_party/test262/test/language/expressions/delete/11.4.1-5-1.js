// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator returns false when deleting a direct reference to
    a var
flags: [noStrict]
---*/

var x = 1;

// Now, deleting 'x' directly should fail;
var d = delete x;

assert.sameValue(d, false, 'd');
assert.sameValue(x, 1, 'x');
