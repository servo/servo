// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.asyncdispose
description: Well-known symbols do not have a key in the global registry
features: [cross-realm, explicit-resource-management]
---*/

assert.sameValue(Symbol.keyFor(Symbol.asyncDispose), undefined);
