// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Replacer function is called with correct context and arguments.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  1. Let value be ? Get(holder, key).
  [...]
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
includes: [compareArray.js]
---*/

var calls = [];
var replacer = function(key, value) {
  if (key !== '') {
    calls.push([this, key, value]);
  }

  return value;
};

var b1 = [1, 2];
var b2 = {c1: true, c2: false};
var a1 = {
  b1: b1,
  b2: {
    toJSON: function() { return b2; },
  },
};
var obj = {a1: a1, a2: 'a2'};

assert.sameValue(
  JSON.stringify(obj, replacer),
  JSON.stringify(obj)
);

assert.compareArray(calls[0], [obj, 'a1', a1]);
assert.compareArray(calls[1], [a1, 'b1', b1]);
assert.compareArray(calls[2], [b1, '0', 1]);
assert.compareArray(calls[3], [b1, '1', 2]);
assert.compareArray(calls[4], [a1, 'b2', b2]);
assert.compareArray(calls[5], [b2, 'c1', true]);
assert.compareArray(calls[6], [b2, 'c2', false]);
assert.compareArray(calls[7], [obj, 'a2', 'a2']);
