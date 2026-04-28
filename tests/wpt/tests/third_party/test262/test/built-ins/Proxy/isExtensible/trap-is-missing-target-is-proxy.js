// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-isextensible
description: >
  If "isExtensible" trap is null or undefined, [[IsExtensible]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[IsExtensible]] ( )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Let trap be ? GetMethod(handler, "isExtensible").
  6. If trap is undefined, then
    a. Return ? IsExtensible(target).

  IsExtensible ( O )

  1. Assert: Type(O) is Object.
  2. Return ? O.[[IsExtensible]]().
features: [Proxy]
---*/

var regExp = /(?:)/g;
Object.preventExtensions(regExp);

var regExpTarget = new Proxy(regExp, {});
var regExpProxy = new Proxy(regExpTarget, {});

assert(!Object.isExtensible(regExpProxy));
