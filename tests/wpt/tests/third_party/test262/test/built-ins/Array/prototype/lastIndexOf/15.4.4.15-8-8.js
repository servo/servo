// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf must return correct index (Array)
---*/

var b = new Array("0,1");
var a = new Array(0, b, "0,1", 3);

assert.sameValue(a.lastIndexOf(b.toString()), 2, 'a.lastIndexOf(b.toString())');
assert.sameValue(a.lastIndexOf("0,1"), 2, 'a.lastIndexOf("0,1")');
