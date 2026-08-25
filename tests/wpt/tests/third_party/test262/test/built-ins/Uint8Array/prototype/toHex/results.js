// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tohex
description: Conversion of Uint8Arrays to hex strings
features: [uint8array-base64, TypedArray]
---*/

assert.sameValue((new Uint8Array([])).toHex(), "");
assert.sameValue((new Uint8Array([102])).toHex(), "66");
assert.sameValue((new Uint8Array([102, 111])).toHex(), "666f");
assert.sameValue((new Uint8Array([102, 111, 111])).toHex(), "666f6f");
assert.sameValue((new Uint8Array([102, 111, 111, 98])).toHex(), "666f6f62");
assert.sameValue((new Uint8Array([102, 111, 111, 98, 97])).toHex(), "666f6f6261");
assert.sameValue((new Uint8Array([102, 111, 111, 98, 97, 114])).toHex(), "666f6f626172");
