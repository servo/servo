// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some applied to applied to string primitive
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof String;
}

assert(Array.prototype.some.call("hello\nw_orld\\!", callbackfn), 'Array.prototype.some.call("hello\nw_orld\\!", callbackfn) !== true');
