// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: BigInt toJSON method
esid: sec-serializejsonproperty
info: |
  Runtime Semantics: SerializeJSONProperty ( key, holder )

  2. If Type(value) is Object or BigInt, then
    a. Let toJSON be ? GetGetV(value, "toJSON").
    b. If IsCallable(toJSON) is true, then
      i. Set value to ? Call(toJSON, value, « key »).
features: [BigInt]
---*/

BigInt.prototype.toJSON = function () { return this.toString(); };
assert.sameValue(JSON.stringify(0n), '"0"');
