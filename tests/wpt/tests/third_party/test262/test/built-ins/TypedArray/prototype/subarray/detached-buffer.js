// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Throws a TypeError creating a new instance with a detached buffer
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  7. Let relativeBegin be ? ToInteger(begin).
  ...
  9. If end is undefined, let relativeEnd be srcLength; else, let relativeEnd be
  ? ToInteger(end).
  ...
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).

  22.2.4.7 TypedArraySpeciesCreate ( exemplar, argumentList )

  ...
  3. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
  4. Return ? TypedArrayCreate(constructor, argumentList).

  22.2.4.6 TypedArrayCreate ( constructor, argumentList )

  1. Let newTypedArray be ? Construct(constructor, argumentList).
  ...

  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  ...
  11. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

var begin, end;

var o1 = {
  valueOf: function() {
    begin = true;
    return 0;
  }
};

var o2 = {
  valueOf: function() {
    end = true;
    return 2;
  }
};

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA(2);
  begin = false;
  end = false;

  $DETACHBUFFER(sample.buffer);
  assert.throws(TypeError, function() {
    sample.subarray(o1, o2);
  });

  assert(begin, "observable ToInteger(begin)");
  assert(end, "observable ToInteger(end)");
}, null, ["passthrough"]);
