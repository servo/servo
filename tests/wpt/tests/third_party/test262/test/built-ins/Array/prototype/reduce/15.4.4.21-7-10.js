// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - 'initialValue' is present
---*/

var str = "initialValue is present";

assert.sameValue([].reduce(function() {}, str), str, '[].reduce(function () { }, str)');
