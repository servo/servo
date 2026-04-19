// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - 'initialValue' is not present
---*/

var str = "initialValue is not present";

assert.sameValue([str].reduce(function() {}), str, '[str].reduce(function () { })');
