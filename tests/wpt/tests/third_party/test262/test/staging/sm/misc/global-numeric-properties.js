/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  undefined, Infinity, and NaN global properties should not be writable
info: bugzilla.mozilla.org/show_bug.cgi?id=537863
esid: pending
---*/

var desc, old;
var global = this;

var names = ["NaN", "Infinity", "undefined"];

for (var i = 0; i < names.length; i++)
{
  var name = names[i];
  desc = Object.getOwnPropertyDescriptor(global, name);
  assert.sameValue(desc !== undefined, true, name + " should be present");
  assert.sameValue(desc.enumerable, false, name + " should not be enumerable");
  assert.sameValue(desc.configurable, false, name + " should not be configurable");
  assert.sameValue(desc.writable, false, name + " should not be writable");

  old = global[name];
  global[name] = 17;
  assert.sameValue(global[name], old, name + " changed on setting?");

  assert.throws(TypeError, function() {
    "use strict";
    global[name] = 42;
  }, "wrong strict mode error setting " + name);
}
