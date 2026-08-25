// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - thisArg not passed to strict callbackfn
flags: [noStrict]
---*/

var innerThisCorrect = false;

function callbackfn(val, idx, obj) {
  "use strict";
  innerThisCorrect = this === undefined;
  return true;
}

[1].forEach(callbackfn);

assert(innerThisCorrect, 'innerThisCorrect !== true');
