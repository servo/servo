// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.valueof
description: valueOf called with a BigInt object from another realm
features: [BigInt, cross-realm]
---*/

var other = $262.createRealm().global;
var wrapped = other.Object(other.BigInt(0));

assert.sameValue(BigInt.prototype.valueOf.call(wrapped), 0n,
                 "cross-realm valueOf");
