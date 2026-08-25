// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator returns false when deleting a direct reference to
    a function name
flags: [noStrict]
---*/

var foo = function() {};

// Now, deleting 'foo' directly should fail;
var d = delete foo;

assert.sameValue(d, false, 'd');
assert.sameValue(typeof foo, 'function', 'typeof foo');
