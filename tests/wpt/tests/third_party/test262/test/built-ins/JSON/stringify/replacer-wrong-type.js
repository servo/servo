// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Replacer paramter of wrong type is silently ignored.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
    a. If IsCallable(replacer) is true, then
      i. Set ReplacerFunction to replacer.
    b. Else,
      i. Let isArray be ? IsArray(replacer).
      ii. If isArray is true, then
        1. Set PropertyList to a new empty List.
features: [Symbol]
---*/

var obj = {key: [1]};
var json = '{"key":[1]}';

assert.sameValue(JSON.stringify(obj, {}), json);
assert.sameValue(JSON.stringify(obj, new String('str')), json);
assert.sameValue(JSON.stringify(obj, new Number(6.1)), json);

assert.sameValue(JSON.stringify(obj, null), json);
assert.sameValue(JSON.stringify(obj, ''), json);
assert.sameValue(JSON.stringify(obj, 0), json);
assert.sameValue(JSON.stringify(obj, Symbol()), json);
assert.sameValue(JSON.stringify(obj, true), json);
