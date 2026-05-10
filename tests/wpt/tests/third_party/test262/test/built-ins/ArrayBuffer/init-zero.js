// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer-length
description: All bytes are initialized to zero
info: |
    [...]
    5. Return ? AllocateArrayBuffer(NewTarget, byteLength).

    24.1.1.1 AllocateArrayBuffer

    3. Let block be ? CreateByteDataBlock(byteLength).

    6.2.6.1 CreateByteDataBlock

    1. Assert: size≥0.
    2. Let db be a new Data Block value consisting of size bytes. If it is
       impossible to create such a Data Block, throw a RangeError exception.
    3. Set all of the bytes of db to 0.
    4. Return db. 
features: [DataView]
---*/

var view = new DataView(new ArrayBuffer(9));

assert.sameValue(view.getUint8(0), 0, 'index 0');
assert.sameValue(view.getUint8(1), 0, 'index 1');
assert.sameValue(view.getUint8(2), 0, 'index 2');
assert.sameValue(view.getUint8(3), 0, 'index 3');
assert.sameValue(view.getUint8(4), 0, 'index 4');
assert.sameValue(view.getUint8(5), 0, 'index 5');
assert.sameValue(view.getUint8(6), 0, 'index 6');
assert.sameValue(view.getUint8(7), 0, 'index 7');
assert.sameValue(view.getUint8(8), 0, 'index 8');
