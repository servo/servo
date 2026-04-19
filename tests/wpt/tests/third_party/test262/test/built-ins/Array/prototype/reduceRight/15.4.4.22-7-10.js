// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight - 'initialValue' is present
---*/

var str = "initialValue is present";

assert.sameValue([].reduceRight(function() {}, str), str, '[].reduceRight(function () { }, str)');
