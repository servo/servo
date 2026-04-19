// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.2.1.1
description: >
    Proxy ( target, handler )

    When Proxy is called with arguments target and handler performs
    the following steps:

    ...
    2. Return ProxyCreate(target, handler). (9.5.15)
    ...
        9.5.15 ProxyCreate(target, handler)
        ...
        5. Let P be a newly created object.
        ...
        10. Return P.

features: [Proxy]
---*/

var p1 = new Proxy({}, {});

assert.sameValue(
  typeof p1,
  'object',
  'Return a newly created Object'
);
