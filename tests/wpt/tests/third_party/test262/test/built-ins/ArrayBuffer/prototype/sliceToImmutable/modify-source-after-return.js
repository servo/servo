// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: Unaffected by post-hoc manipulation of the source
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  ...
  15. Let newBuffer be ? AllocateImmutableArrayBuffer(%ArrayBuffer%, newLen, fromBuf, first, newLen).
  16. Return newBuffer.
features: [immutable-arraybuffer]
includes: [compareArray.js, detachArrayBuffer.js]
---*/

var source = new ArrayBuffer(8, { maxByteLength: 8 });
var view = new Uint8Array(source);
for (var i = 0; i < 8; i++) view[i] = i + 1;

var dest = source.sliceToImmutable();
var expectContents = [1, 2, 3, 4, 5, 6, 7, 8];
var destView = new Uint8Array(dest);
assert.compareArray(destView, expectContents);

view[0] = 86;
assert.compareArray(destView, expectContents, "after source overwrite");
assert.compareArray(new Uint8Array(dest), expectContents, "new view after source overwrite");

if (source.resize) {
  source.resize(4);
  assert.compareArray(destView, expectContents, "after resize");
  assert.compareArray(new Uint8Array(dest), expectContents, "new view after resize");
}

$DETACHBUFFER(source);
assert.compareArray(destView, expectContents, "after detach");
assert.compareArray(new Uint8Array(dest), expectContents, "new view after detach");
