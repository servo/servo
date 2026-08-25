// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Abrupt completions from Get and Call.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  2. If Type(value) is Object, then
    a. Let toJSON be ? Get(value, "toJSON").
    b. If IsCallable(toJSON) is true, then
      i. Set value to ? Call(toJSON, value, « key »).
---*/

assert.throws(Test262Error, function() {
  JSON.stringify({
    get toJSON() {
      throw new Test262Error();
    },
  });
});

assert.throws(Test262Error, function() {
  JSON.stringify({
    toJSON() {
      throw new Test262Error();
    },
  });
});
