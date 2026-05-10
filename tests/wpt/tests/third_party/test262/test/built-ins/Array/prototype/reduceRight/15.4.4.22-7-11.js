// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight - 'initialValue' is not present
---*/

var str = "initialValue is not present";

assert.sameValue([str].reduceRight(function() {}), str, '[str].reduceRight(function () { })');
