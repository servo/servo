// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-serializejsonproperty
description: JSON.stringify called with a BigInt object from another realm
features: [BigInt, cross-realm]
---*/

var other = $262.createRealm().global;
var wrapped = other.Object(other.BigInt(100));

assert.throws(TypeError, () => JSON.stringify(wrapped),
              "cross-realm BigInt object without toJSON method");

other.BigInt.prototype.toJSON = function () { return this.toString(); };

assert.sameValue(JSON.stringify(wrapped), "\"100\"",
                 "cross-realm BigInt object with toJSON method");
