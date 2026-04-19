// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - undefined passed as thisValue to
    strict callbackfn
flags: [noStrict]
---*/

var innerThisCorrect = false;

function callbackfn(prevVal, curVal, idx, obj)
{
  "use strict";
  innerThisCorrect = this === undefined;
  return true;
}
[0].reduceRight(callbackfn, true);

assert(innerThisCorrect, 'innerThisCorrect !== true');
