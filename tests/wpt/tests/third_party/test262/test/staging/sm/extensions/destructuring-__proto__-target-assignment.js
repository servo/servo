/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  { __proto__: target } should work as a destructuring assignment pattern
info: bugzilla.mozilla.org/show_bug.cgi?id=963641
esid: pending
---*/

function objectWithProtoProperty(v)
{
  var obj = {};
  return Object.defineProperty(obj, "__proto__",
                               {
                                 enumerable: true,
                                 configurable: true,
                                 writable: true,
                                 value: v
                               });
}

var { __proto__: target } = objectWithProtoProperty(null);
assert.sameValue(target, null);

({ __proto__: target } = objectWithProtoProperty("aacchhorrt"));
assert.sameValue(target, "aacchhorrt");

function nested()
{
  var { __proto__: target } = objectWithProtoProperty(3.141592654);
  assert.sameValue(target, 3.141592654);

  ({ __proto__: target } = objectWithProtoProperty(-0));
  assert.sameValue(target, -0);
}
nested();
