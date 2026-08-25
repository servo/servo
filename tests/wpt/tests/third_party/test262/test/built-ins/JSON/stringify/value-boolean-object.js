// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Boolean objects are converted to primitives using [[BooleanData]].
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  4. If Type(value) is Object, then
    [...]
    c. Else if value has a [[BooleanData]] internal slot, then
      i. Set value to value.[[BooleanData]].
  [...]
  6. If value is true, return "true".
  7. If value is false, return "false".
---*/

assert.sameValue(JSON.stringify(new Boolean(true)), 'true');

assert.sameValue(
  JSON.stringify({
    toJSON: function() {
      return {key: new Boolean(false)};
    },
  }),
  '{"key":false}'
);

assert.sameValue(JSON.stringify([1], function(_k, v) {
  return v === 1 ? new Boolean(true) : v;
}), '[true]');
