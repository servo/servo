// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-length
description: All bytes are initialized to zero
info: |
  22.2.4.2 TypedArray ( length )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is not Object.

  ...
  8. Return ? AllocateTypedArray(constructorName, NewTarget,
  %TypedArrayPrototype%, elementLength).

  22.2.4.2.1 Runtime Semantics: AllocateTypedArray (constructorName, newTarget,
  defaultProto [ , length ])

  5. If length was not passed, then
     ...
  6. Else,
     a. Perform ? AllocateTypedArrayBuffer(obj, length).

  22.2.4.2.2 Runtime Semantics: AllocateTypedArrayBuffer

  7. Let data be ? AllocateArrayBuffer(%ArrayBuffer%, byteLength).

  24.1.1.1 AllocateArrayBuffer

  3. Let block be ? CreateByteDataBlock(byteLength).

  6.2.6.1 CreateByteDataBlock

  1. Assert: size≥0.
  2. Let db be a new Data Block value consisting of size bytes. If it is
     impossible to create such a Data Block, throw a RangeError exception.
  3. Set all of the bytes of db to 0.
  4. Return db. 
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var subject = new TA(makeCtorArg(9));

  assert.sameValue(subject[0], 0, 'index 0');
  assert.sameValue(subject[1], 0, 'index 1');
  assert.sameValue(subject[2], 0, 'index 2');
  assert.sameValue(subject[3], 0, 'index 3');
  assert.sameValue(subject[4], 0, 'index 4');
  assert.sameValue(subject[5], 0, 'index 5');
  assert.sameValue(subject[6], 0, 'index 6');
  assert.sameValue(subject[7], 0, 'index 7');
  assert.sameValue(subject[8], 0, 'index 8');
});
