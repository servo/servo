// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Return value when no match is found
info: |
    [...]
    13. If result is null, return –1.
features: [Symbol.search]
---*/

assert.sameValue(/z/[Symbol.search]('a'), -1);
