// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-escape-string
es6id: B.2.1.1
description: Input is the empty string
info: |
    1. Let string be ? ToString(string).
    2. Let length be the number of code units in string.
    3. Let R be the empty string.
    4. Let k be 0.
    5. Repeat, while k < length,
       [...]
    6. Return R.
---*/

assert.sameValue(escape(''), '');
