// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 26.2.2.1.1
description: The [[Extensible]] slot of Proxy Revocation functions
info: |
  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
features: [Proxy]
---*/

var revocationFunction = Proxy.revocable({}, {}).revoke;

assert(Object.isExtensible(revocationFunction));
