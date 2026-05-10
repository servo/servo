// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-set-p-v-receiver
description: >
  Returns false if key has a numeric index and object has a detached buffer
info: |
  [[Set]] ( P, V, Receiver)

  ...
   If Type(P) is String, then
    Let numericIndex be ! CanonicalNumericIndexString(P).
    If numericIndex is not undefined, then
      Return ? IntegerIndexedElementSet(O, numericIndex, V).
  ...

  IntegerIndexedElementSet ( O, index, value )

  Assert: O is an Integer-Indexed exotic object.
  Assert: Type(index) is Number.
  If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
  Otherwise, let numValue be ? ToNumber(value).
  Let buffer be O.[[ViewedArrayBuffer]].
  If IsDetachedBuffer(buffer) is true, return false.

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray]
---*/
testWithTypedArrayConstructors(function(TA) {
  let sample = new TA([42]);
  $DETACHBUFFER(sample.buffer);
  sample[0] = 1;

  assert.sameValue(sample[0], undefined, '`sample[0] = 1` is undefined');
  sample['1.1'] = 1;
  assert.sameValue(sample['1.1'], undefined, '`sample[\'1.1\'] = 1` is undefined');
  sample['-0'] = 1;
  assert.sameValue(sample['-0'], undefined, '`sample[\'-0\'] = 1` is undefined');
  sample['-1'] = 1;
  assert.sameValue(sample['-1'], undefined, '`sample[\'-1\'] = 1` is undefined');
  sample['1'] = 1;
  assert.sameValue(sample['1'], undefined, '`sample[\'1\'] = 1` is undefined');
  sample['2'] = 1;
  assert.sameValue(sample['2'], undefined, '`sample[\'2\'] = 1` is undefined');

  let obj = {
    valueOf() {
      throw new Test262Error();
    }
  };

  assert.throws(Test262Error, function() {
    sample['0'] = obj;
  });
}, null, ["passthrough"]);
