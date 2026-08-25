// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: >
  Throws a TypeError if _O_.[[ViewedArrayBuffer]] is detached. Using other
  targetType
info: |
  22.2.3.24 %TypedArray%.prototype.slice ( start, end )

  ...
  Let A be ? TypedArraySpeciesCreate(O, « count »).
  If count > 0, then
    If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [align-detached-buffer-semantics-with-web-reality, Symbol.species, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  let counter = 0;
  var sample = new TA(makeCtorArg(1));

  sample.constructor = {};
  sample.constructor[Symbol.species] = function(count) {
    var other = TA === Int8Array ? Int16Array : Int8Array;
    $DETACHBUFFER(sample.buffer);
    counter++;
    return new other(count);
  };

  assert.throws(TypeError, function() {
    counter++;
    sample.slice();
  }, '`sample.slice()` throws TypeError');

  assert.sameValue(counter, 2, 'The value of `counter` is 2');
}, null, null, ["immutable"]);
