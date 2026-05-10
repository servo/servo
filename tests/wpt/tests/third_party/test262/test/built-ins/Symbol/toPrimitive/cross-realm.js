// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.toprimitive
description: Value shared by all realms
info: |
  Unless otherwise specified, well-known symbols values are shared by all
  realms.
features: [cross-realm, Symbol.split]
---*/

var OSymbol = $262.createRealm().global.Symbol;

assert.sameValue(Symbol.split, OSymbol.split);
