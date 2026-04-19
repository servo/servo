// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  toJSON is called with correct context and arguments.
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

var callCount = 0;
var _this, _key;
var obj = {
  toJSON: function(key) {
    callCount += 1;
    _this = this;
    _key = key;
  },
};

assert.sameValue(JSON.stringify(obj), undefined);
assert.sameValue(callCount, 1);
assert.sameValue(_this, obj);
assert.sameValue(_key, '');

assert.sameValue(JSON.stringify([1, obj, 3]), '[1,null,3]');
assert.sameValue(callCount, 2);
assert.sameValue(_this, obj);
assert.sameValue(_key, '1');

assert.sameValue(JSON.stringify({key: obj}), '{}');
assert.sameValue(callCount, 3);
assert.sameValue(_this, obj);
assert.sameValue(_key, 'key');
