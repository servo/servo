// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm-constructor
description: >
  The ShadowRealm constructor is extensible
info: |
  17 ECMAScript Standard Built-in Objects

  Unless specified otherwise, the [[Extensible]] internal slot of a built-in
  object initially has the value true.
features: [ShadowRealm]
---*/

assert.sameValue(Object.isExtensible(ShadowRealm), true);
