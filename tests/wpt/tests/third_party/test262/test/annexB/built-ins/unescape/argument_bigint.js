// Copyright (C) 2020 Qu Xing. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
description: Input is a BigInt
info: |
    B.2.1.2 unescape ( string )

    1. Set string to ? ToString(string).
    ....
features: [BigInt]
---*/

assert.sameValue(unescape(1n), '1');

assert.sameValue(unescape(-1n), '-1');
