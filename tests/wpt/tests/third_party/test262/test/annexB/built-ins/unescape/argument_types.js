// Copyright (C) 2020 Qu Xing. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
description: Input is a null, undefined, boolean or Number
info: |
    B.2.1.2 unescape ( string )

    1. Set string to ? ToString(string).
    ...
---*/

assert.sameValue(unescape(null), 'null');

assert.sameValue(unescape(undefined), 'undefined');

assert.sameValue(unescape(), 'undefined');

assert.sameValue(unescape(true), 'true');

assert.sameValue(unescape(false), 'false');

assert.sameValue(unescape(-0), '0');

assert.sameValue(unescape(0), '0');

assert.sameValue(unescape(1), '1');

assert.sameValue(unescape(NaN), 'NaN');

assert.sameValue(unescape(Number.POSITIVE_INFINITY), 'Infinity');

assert.sameValue(unescape(Number.NEGATIVE_INFINITY), '-Infinity');
