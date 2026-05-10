// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Replacer function is called on result of toJSON method.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
    a. If IsCallable(replacer) is true, then
      i. Let ReplacerFunction be replacer.
  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  2. If Type(value) is Object, then
    a. Let toJSON be ? Get(value, "toJSON").
    b. If IsCallable(toJSON) is true, then
      i. Set value to ? Call(toJSON, value, « key »).
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
---*/

assert.sameValue(
  JSON.stringify({
    toJSON: function() {
      return 'toJSON';
    },
  }, function(_key, value) {
    return value + '|replacer';
  }),
  '"toJSON|replacer"'
);

assert.sameValue(
  JSON.stringify({
    toJSON: function() {
      return {calls: 'toJSON'};
    },
  }, function(_key, value) {
    if (value && value.calls) {
      value.calls += '|replacer';
    }

    return value;
  }),
  '{"calls":"toJSON|replacer"}'
);
