// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
// Test case written by André Bargull.

/*---
esid: sec-serializejsonproperty
description: toJSON method called with BigInt as receiver
features: [BigInt]
---*/

assert.throws(TypeError, () => JSON.stringify(1n),
              "toString throws for BigInt object");

// The BigInt proposal changes the SerializeJSONProperty algorithm to
// specifically allow passing BigInt objects as receivers for the toJSON
// method.
Object.defineProperty(BigInt.prototype, "toJSON", {
    get() {
        "use strict";
        return () => typeof this;
    }
});

assert.sameValue(JSON.stringify(1n), "\"bigint\"",
                 "BigInt toJSON method called with value as receiver");
