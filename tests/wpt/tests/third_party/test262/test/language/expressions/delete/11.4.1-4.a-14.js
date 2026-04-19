// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This test is actually testing the [[Delete]] internal method (8.12.8). Since the
    language provides no way to directly exercise [[Delete]], the tests are placed here.
esid: sec-delete-operator-runtime-semantics-evaluation
description: delete operator returns true when deleting Array elements
---*/

var a = [1, 2, 3];
a.x = 10;
var d = delete a[1];

assert.sameValue(d, true, 'd');
assert.sameValue(a[1], undefined, 'a[1]');
