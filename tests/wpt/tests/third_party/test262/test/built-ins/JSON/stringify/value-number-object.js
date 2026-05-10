// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Number objects are converted to primitives using ToNumber.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  4. If Type(value) is Object, then
    a. If value has a [[NumberData]] internal slot, then
      i. Set value to ? ToNumber(value).
  [...]
  9. If Type(value) is Number, then
    a. If value is finite, return ! ToString(value).
---*/

assert.sameValue(JSON.stringify(new Number(8.5)), '8.5');

var toPrimitiveReplacer = function(_key, value) {
  if (value === 'str') {
    var num = new Number(42);
    num.toString = function() { throw new Test262Error('should not be called'); };
    num.valueOf = function() { return 2; };
    return num;
  }

  return value;
};

assert.sameValue(JSON.stringify(['str'], toPrimitiveReplacer), '[2]');

var abruptToJSON = function() {
  var num = new Number(3.14);
  num.toString = function() { throw new Test262Error(); };
  num.valueOf = function() { throw new Test262Error(); };
  return num;
};

assert.throws(Test262Error, function() {
  JSON.stringify({
    key: {
      toJSON: abruptToJSON,
    },
  });
});
