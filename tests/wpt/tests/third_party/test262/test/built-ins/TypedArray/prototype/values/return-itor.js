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
features: [TypedArray]
---*/

var sample = [0, 42, 64];

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var typedArray = new TA(makeCtorArg(sample));
  var itor = typedArray.values();

  var next = itor.next();
  assert.sameValue(next.value, 0);
  assert.sameValue(next.done, false);

  next = itor.next();
  assert.sameValue(next.value, 42);
  assert.sameValue(next.done, false);

  next = itor.next();
  assert.sameValue(next.value, 64);
  assert.sameValue(next.done, false);

  next = itor.next();
  assert.sameValue(next.value, undefined);
  assert.sameValue(next.done, true);
});
