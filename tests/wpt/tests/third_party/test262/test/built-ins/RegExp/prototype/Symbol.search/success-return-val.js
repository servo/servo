// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Return value following successful match
info: |
    [...]
    14. Return Get(result, "index").
features: [Symbol.search]
---*/

assert.sameValue(/a/[Symbol.search]('abc'), 0);
assert.sameValue(/b/[Symbol.search]('abc'), 1);
assert.sameValue(/c/[Symbol.search]('abc'), 2);
