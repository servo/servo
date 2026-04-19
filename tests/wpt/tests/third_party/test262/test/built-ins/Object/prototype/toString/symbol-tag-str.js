// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    String values of `Symbol.toStringTag` property are honored
es6id: 19.1.3.6
info: |
    16. Let tag be Get (O, @@toStringTag).
    17. ReturnIfAbrupt(tag).
    18. If Type(tag) is not String, let tag be builtinTag.
    19. Return the String that is the result of concatenating "[object ", tag,
        and "]".
features: [Symbol.toStringTag]
---*/

var custom = {};
custom[Symbol.toStringTag] = 'test262';

assert.sameValue(Object.prototype.toString.call(custom), '[object test262]');
