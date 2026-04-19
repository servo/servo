/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  { __proto__ } should work as a destructuring assignment pattern
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

var { __proto__ } = objectWithProtoProperty(42);
assert.sameValue(__proto__, 42);

({ __proto__ } = objectWithProtoProperty(17));
assert.sameValue(__proto__, 17);

function nested()
{
  var { __proto__ } = objectWithProtoProperty("fnord");
  assert.sameValue(__proto__, "fnord");

  ({ __proto__ } = objectWithProtoProperty(undefined));
  assert.sameValue(__proto__, undefined);
}
nested();
