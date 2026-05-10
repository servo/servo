// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-internalizejsonproperty
description: Abrupt completion from defining array property while reviving
info: |
  JSON.parse ( text [ , reviver ] )

  [...]
  7. If IsCallable(reviver) is true, then
     [...]
     e. Return ? InternalizeJSONProperty(root, rootName).

  Runtime Semantics: InternalizeJSONProperty ( holder, name)

  1. Let val be ? Get(holder, name).
  2. If Type(val) is Object, then
     a. Let isArray be ? IsArray(val).
     b. If isArray is true, then
        i. Set I to 0.
        ii. Let len be ? ToLength(? Get(val, "length")).
        iii. Repeat while I < len,
             1. Let newElement be ? InternalizeJSONProperty(val, !
                ToString(I)).
             2. If newElement is undefined, then
                [...]
             3. Else,
                a. Perform ? CreateDataProperty(val, ! ToString(I),
                   newElement).
features: [Proxy]
---*/

var badDefine = new Proxy([null], {
  defineProperty: function(_, name) {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  JSON.parse('["first", null]', function(_, value) {
    if (value === 'first') {
      this[1] = badDefine;
    }
    return value;
  });
});
