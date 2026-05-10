// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Result of toJSON method is stringified.
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

assert.sameValue(
  JSON.stringify({
    toJSON: function() { return [false]; },
  }),
  '[false]'
);

var arr = [true];
arr.toJSON = function() {};
assert.sameValue(JSON.stringify(arr), undefined);

var str = new String('str');
str.toJSON = function() { return null; };
assert.sameValue(JSON.stringify({key: str}), '{"key":null}');

var num = new Number(14);
num.toJSON = function() { return {key: 7}; };
assert.sameValue(JSON.stringify([num]), '[{"key":7}]');
