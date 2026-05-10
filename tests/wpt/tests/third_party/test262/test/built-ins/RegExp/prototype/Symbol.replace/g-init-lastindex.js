// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Initialization of `lastIndex` property for "global" instances
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    10. If global is true, then
        [...]
        c. Let setStatus be Set(rx, "lastIndex", 0, true).
        [...]
features: [Symbol.replace]
---*/

var r = /./g;

r.lastIndex = 1;

assert.sameValue(r[Symbol.replace]('aa', 'x'), 'xx');
