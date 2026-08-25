// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Value shared by all realms
info: |
  Unless otherwise specified, well-known symbols values are shared by all
  realms.
features: [cross-realm, Symbol.matchAll]
---*/

var OSymbol = $262.createRealm().global.Symbol;

assert.sameValue(Symbol.matchAll, OSymbol.matchAll);
