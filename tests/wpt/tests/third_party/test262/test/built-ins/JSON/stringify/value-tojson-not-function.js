// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  toJSON value is not callable.
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

assert.sameValue(JSON.stringify({toJSON: null}), '{"toJSON":null}');
assert.sameValue(JSON.stringify({toJSON: false}), '{"toJSON":false}');
assert.sameValue(JSON.stringify({toJSON: []}), '{"toJSON":[]}');
assert.sameValue(JSON.stringify({toJSON: /re/}), '{"toJSON":{}}');
