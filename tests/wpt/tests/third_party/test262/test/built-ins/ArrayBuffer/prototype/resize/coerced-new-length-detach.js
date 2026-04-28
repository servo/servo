// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.resize
description: >
  ArrayBuffer.p.resize has one detach check after argument coercion
includes: [detachArrayBuffer.js]
features: [resizable-arraybuffer]
---*/

{
  const rab = new ArrayBuffer(64, { maxByteLength: 1024 });
  let called = false;
  assert.throws(TypeError, () => rab.resize({ valueOf() {
    $DETACHBUFFER(rab);
    called = true;
  }}));
  assert(called);
}

{
  const rab = new ArrayBuffer(64, { maxByteLength: 1024 });
  $DETACHBUFFER(rab);
  let called = false;
  assert.throws(TypeError, () => rab.resize({ valueOf() {
    called = true;
  }}));
  assert(called);
}
