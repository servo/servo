// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Result of replacer function is stringified.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
---*/

assert.sameValue(JSON.stringify(1, function() {}), undefined);
assert.sameValue(JSON.stringify([1], function() {}), undefined);
assert.sameValue(JSON.stringify({prop: 1}, function() {}), undefined);

var replacer = function(_key, value) {
  return value === 1 ? undefined : value;
};

assert.sameValue(JSON.stringify([1], replacer), '[null]');
assert.sameValue(JSON.stringify({prop: 1}, replacer), '{}');
assert.sameValue(JSON.stringify({
  a: {
    b: [1],
  },
}, replacer), '{"a":{"b":[null]}}');
