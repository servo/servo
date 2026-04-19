// Copyright (C) 2020 Qu Xing. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-escape-string
description: Input is a BigInt
info: |
    B.2.1.1 escape ( string )

    1. Let string be ? ToString(string).
    ...
features: [BigInt]
---*/

assert.sameValue(escape(1n), '1');

assert.sameValue(escape(-1n), '-1');
