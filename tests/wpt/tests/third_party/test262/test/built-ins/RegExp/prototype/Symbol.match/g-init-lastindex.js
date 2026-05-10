// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Initializing the value of `lastIndex` on "global" instances
es6id: 21.2.5.6
info: |
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       c. Let setStatus be Set(rx, "lastIndex", 0, true).
features: [Symbol.match]
---*/

var r = /./g;

r.lastIndex = 1;

assert.notSameValue(r[Symbol.match]('a'), null);
