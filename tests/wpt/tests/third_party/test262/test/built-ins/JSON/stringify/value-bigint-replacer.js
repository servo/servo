// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: JSON serialization of BigInt values with replacer
esid: sec-serializejsonproperty
info: |
  Runtime Semantics: SerializeJSONProperty ( key, holder )

  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
features: [BigInt]
---*/

function replacer(k, v)
{
    if (typeof v === "bigint")
        return "bigint";
    else
        return v;
}

assert.sameValue(JSON.stringify(0n, replacer), '"bigint"');
assert.sameValue(JSON.stringify({x: 0n}, replacer), '{"x":"bigint"}');
