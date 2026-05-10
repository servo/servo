// Copyright (C) 2020 Rick Waldron. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-delete-p
description: >
  Throws TypeError exception in strict mode.
flags: [onlyStrict]
includes: [testTypedArray.js]
features: [align-detached-buffer-semantics-with-web-reality, TypedArray, SharedArrayBuffer]
---*/

testWithTypedArrayConstructors(function(TA) {
  let proto = TypedArray.prototype;
  let descriptorGetterThrows = {
    configurable: true,
    get() {
      throw new Test262Error("OrdinaryGet was called!");
    }
  };
  Object.defineProperties(proto, {
    ["0"]: descriptorGetterThrows,
    ["1"]: descriptorGetterThrows,
  });
  let sab = new SharedArrayBuffer(TA.BYTES_PER_ELEMENT * 2);
  let sample = new TA(sab);

  assert.throws(TypeError, () => {
    delete sample["0"];
  });
  assert.throws(TypeError, () => {
    delete sample[0];
  });
  assert.throws(TypeError, () => {
    delete sample["1"];
  });
  assert.throws(TypeError, () => {
    delete sample[1];
  });
}, null, ["passthrough"]);
