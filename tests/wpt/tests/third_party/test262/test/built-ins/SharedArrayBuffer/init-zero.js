// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createsharedbytedatablock
description: All bytes are initialized to zero
features: [SharedArrayBuffer, DataView]
---*/

var view = new DataView(new SharedArrayBuffer(9));

assert.sameValue(view.getUint8(0), 0, 'index 0');
assert.sameValue(view.getUint8(1), 0, 'index 1');
assert.sameValue(view.getUint8(2), 0, 'index 2');
assert.sameValue(view.getUint8(3), 0, 'index 3');
assert.sameValue(view.getUint8(4), 0, 'index 4');
assert.sameValue(view.getUint8(5), 0, 'index 5');
assert.sameValue(view.getUint8(6), 0, 'index 6');
assert.sameValue(view.getUint8(7), 0, 'index 7');
assert.sameValue(view.getUint8(8), 0, 'index 8');
