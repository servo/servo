// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Function values are ignored.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    [...]
  11. Return undefined.
---*/

assert.sameValue(JSON.stringify(function() {}), undefined);
assert.sameValue(JSON.stringify([function() {}]), '[null]');
assert.sameValue(JSON.stringify({key: function() {}}), '{}');
