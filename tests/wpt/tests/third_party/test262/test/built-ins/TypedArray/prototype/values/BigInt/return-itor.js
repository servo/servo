// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.values
description: Return an iterator for the values.
info: |
  22.2.3.30 %TypedArray%.prototype.values ( )

  ...
  3. Return CreateArrayIterator(O, "value").
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var typedArray = new TA(makeCtorArg([0n, 42n, 64n]));
  var itor = typedArray.values();

  var next = itor.next();
  assert.sameValue(next.value, 0n);
  assert.sameValue(next.done, false);

  next = itor.next();
  assert.sameValue(next.value, 42n);
  assert.sameValue(next.done, false);

  next = itor.next();
  assert.sameValue(next.value, 64n);
  assert.sameValue(next.done, false);

  next = itor.next();
  assert.sameValue(next.value, undefined);
  assert.sameValue(next.done, true);
});
