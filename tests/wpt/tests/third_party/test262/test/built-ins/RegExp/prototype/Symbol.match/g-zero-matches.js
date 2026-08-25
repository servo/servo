// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Return value when no matches occur with the `global` flag
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          ii. ReturnIfAbrupt(result).
          iii. If result is null, then
               1. If n=0, return null.
features: [Symbol.match]
---*/

assert.sameValue(/a/g[Symbol.match]('b'), null);
