// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-symbol.asynciterator
description: Value shared by all realms
info: |
  Unless otherwise specified, well-known symbols values are shared by all
  realms.
features: [cross-realm, Symbol.asyncIterator]
---*/

var OSymbol = $262.createRealm().global.Symbol;

assert.sameValue(Symbol.asyncIterator, OSymbol.asyncIterator);
