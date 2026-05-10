// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Abrupt completion from Call.
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
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
---*/

assert.throws(Test262Error, function() {
  JSON.stringify({}, function() {
    throw new Test262Error();
  });
});
