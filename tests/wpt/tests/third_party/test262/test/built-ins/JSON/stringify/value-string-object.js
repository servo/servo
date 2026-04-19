// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  String exotic objects are converted to primitives using ToString.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  4. If Type(value) is Object, then
    [...]
    b. Else if value has a [[StringData]] internal slot, then
      i. Set value to ? ToString(value).
  [...]
  8. If Type(value) is String, return QuoteJSONString(value).
---*/

assert.sameValue(JSON.stringify(new String('str')), '"str"');

var toJSON = function() {
  var str = new String('str');
  str.toString = function() { return 'toString'; };
  str.valueOf = function() { throw new Test262Error('should not be called'); };
  return str;
};

assert.sameValue(
  JSON.stringify({
    key: {
      toJSON: toJSON,
    },
  }),
  '{"key":"toString"}'
);

var abruptReplacer = function(_key, value) {
  if (value === true) {
    var str = new String('str');
    str.toString = function() { throw new Test262Error(); };
    str.valueOf = function() { throw new Test262Error(); };
    return str;
  }

  return value;
};

assert.throws(Test262Error, function() {
  JSON.stringify([true], abruptReplacer);
});
