// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: BigInt stringify order of steps
esid: sec-serializejsonproperty
info: |
  Runtime Semantics: SerializeJSONProperty ( key, holder )

  2. If Type(value) is Object or BigInt, then
    a. Let toJSON be ? GetGetV(value, "toJSON").
    b. If IsCallable(toJSON) is true, then
      i. Set value to ? Call(toJSON, value, « key »).
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
  4. If Type(value) is Object, then
    [...]
    d. Else if value has a [[BigIntData]] internal slot, then
      i. Set value to value.[[BigIntData]].
  [...]
  10. If Type(value) is BigInt, throw a TypeError exception
features: [BigInt, arrow-function]
---*/

let step;

function replacer(x, k, v)
{
  assert.sameValue(step++, 1);
  assert.sameValue(v, 1n);
  return x;
}

BigInt.prototype.toJSON = function () { assert.sameValue(step++, 0); return 1n; };

step = 0;
assert.throws(TypeError, () => JSON.stringify(0n, (k, v) => replacer(2n, k, v)));
assert.sameValue(step, 2);

step = 0;
assert.throws(TypeError, () => JSON.stringify(0n, (k, v) => replacer(Object(2n), k, v)));
assert.sameValue(step, 2);
