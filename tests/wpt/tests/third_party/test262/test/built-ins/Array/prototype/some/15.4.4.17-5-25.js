// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - thisArg not passed
flags: [noStrict]
---*/

function innerObj() {
  this._15_4_4_17_5_25 = true;
  var _15_4_4_17_5_25 = false;

  function callbackfn(val, idx, obj) {
    return this._15_4_4_17_5_25;
  }
  var arr = [1];
  this.retVal = !arr.some(callbackfn);
}

assert(new innerObj().retVal, 'new innerObj().retVal !== true');
