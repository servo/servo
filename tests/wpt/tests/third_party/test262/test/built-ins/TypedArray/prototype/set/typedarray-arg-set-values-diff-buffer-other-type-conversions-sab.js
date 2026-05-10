// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set converted values from different buffer of different types and different type instances
includes: [byteConversionValues.js, testTypedArray.js]
features: [SharedArrayBuffer]
---*/

testTypedArrayConversions(byteConversionValues, function(TA, value, expected, initial) {
  if (TA === Float64Array || TA === Float32Array || (typeof Float16Array !== 'undefined' && TA === Float16Array) || TA === Uint8ClampedArray) {
    return;
  }
  if (TA === Int32Array) {
    return;
  }

  var sab, src, target;

  sab = new SharedArrayBuffer(4);
  src = new Int32Array(sab);
  src[0] = value;
  target = new TA([initial]);

  target.set(src);

  assert.sameValue(target[0], expected, "src is SAB-backed");

  sab = new SharedArrayBuffer(4);
  src = new Int32Array([value]);
  target = new TA(sab);
  target[0] = initial;

  target.set(src);

  assert.sameValue(target[0], expected, "target is SAB-backed");

  var sab1 = new SharedArrayBuffer(4);
  var sab2 = new SharedArrayBuffer(4);
  src = new Int32Array(sab1);
  src[0] = value;
  target = new TA(sab2);
  target[0] = initial;

  target.set(src);

  assert.sameValue(target[0], expected, "src and target are SAB-backed");
});
