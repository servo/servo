// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Return value from [[ByteLength]] internal slot
features: [SharedArrayBuffer]
---*/

var ab1 = new SharedArrayBuffer(0);
assert.sameValue(ab1.byteLength, 0);

var ab2 = new SharedArrayBuffer(42);
assert.sameValue(ab2.byteLength, 42);
