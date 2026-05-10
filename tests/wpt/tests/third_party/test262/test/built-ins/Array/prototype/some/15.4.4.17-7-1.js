// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some considers new elements added to array after
    it is called
---*/

var calledForThree = false;

function callbackfn(val, idx, obj)
{
  arr[2] = 3;
  if (val !== 3)
    calledForThree = true;

  return false;
}

var arr = [1, 2, , 4, 5];

var val = arr.some(callbackfn);

assert(calledForThree, 'calledForThree !== true');
