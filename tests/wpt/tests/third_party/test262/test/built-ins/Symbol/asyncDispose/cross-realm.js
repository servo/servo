// Copyright (C) 2025 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.asyncdispose
description: Value shared by all realms
info: |
  Unless otherwise specified, well-known symbols values are shared by all
  realms.
features: [cross-realm, explicit-resource-management]
---*/

var OSymbol = $262.createRealm().global.Symbol;

assert.sameValue(Symbol.asyncDispose, OSymbol.asyncDispose);
