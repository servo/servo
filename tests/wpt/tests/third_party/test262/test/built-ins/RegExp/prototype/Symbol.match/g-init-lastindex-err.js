// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when `lastIndex` cannot be set on "global" instances
es6id: 21.2.5.6
info: |
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       c. Let setStatus be Set(rx, "lastIndex", 0, true).
       d. ReturnIfAbrupt(setStatus).
features: [Symbol.match]
---*/

var r = /./g;
Object.defineProperty(r, 'lastIndex', { writable: false });

assert.throws(TypeError, function() {
  r[Symbol.match]('');
});
