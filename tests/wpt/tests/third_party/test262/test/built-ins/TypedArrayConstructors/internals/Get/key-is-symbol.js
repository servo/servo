// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Use OrdinaryGet if key is a Symbol
info: |
  9.4.5.4 [[Get]] (P, Receiver)

  ...
  2. If Type(P) is String, then
    ...
  3. Return ? OrdinaryGet(O, P, Receiver).
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, Symbol, TypedArray]
---*/

var parentKey = Symbol("2");
TypedArray.prototype[parentKey] = "test262";

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42]));

  var s1 = Symbol("1");

  assert.sameValue(
    sample[s1], undefined,
    "return undefined if not property is present"
  );

  sample[s1] = "foo";
  assert.sameValue(sample[s1], "foo", "return value");

  Object.defineProperty(sample, s1, {
    get: function() { return "bar"; }
  });
  assert.sameValue(sample[s1], "bar", "return value from get accessor");

  assert.sameValue(sample[parentKey], "test262", "value from parent key");
}, null, ["passthrough"]);
