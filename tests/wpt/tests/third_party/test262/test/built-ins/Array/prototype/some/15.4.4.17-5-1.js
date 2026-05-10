// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - thisArg is passed
flags: [noStrict]
---*/

(function() {
  this._15_4_4_17_5_1 = false;
  var _15_4_4_17_5_1 = true;

  function callbackfn(val, idx, obj) {
    return this._15_4_4_17_5_1;
  }
  var arr = [1];

  assert.sameValue(arr.some(callbackfn), false, 'arr.some(callbackfn)');
})();
