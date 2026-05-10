// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - thisArg is passed
flags: [noStrict]
---*/

(function() {
  this._15_4_4_18_5_1 = false;
  var _15_4_4_18_5_1 = true;
  var result;

  function callbackfn(val, idx, obj) {
    result = this._15_4_4_18_5_1;
  }
  var arr = [1];
  arr.forEach(callbackfn)

  assert.sameValue(result, false, 'result');
})();
