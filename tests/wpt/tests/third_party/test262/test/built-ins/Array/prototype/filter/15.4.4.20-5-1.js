// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - thisArg is passed
flags: [noStrict]
---*/

(function() {
  this._15_4_4_20_5_1 = false;
  var _15_4_4_20_5_1 = true;

  function callbackfn(val, idx, obj) {
    return this._15_4_4_20_5_1;
  }
  var srcArr = [1];
  var resArr = srcArr.filter(callbackfn);

  assert.sameValue(resArr.length, 0, 'resArr.length');
})();
