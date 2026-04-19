// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - string primitive can be used as thisArg
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (this.valueOf() === "abc");
}

[11].forEach(callbackfn, "abc");

assert(result, 'result !== true');
