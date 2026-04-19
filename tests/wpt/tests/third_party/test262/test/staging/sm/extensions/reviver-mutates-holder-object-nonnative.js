/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Behavior when JSON.parse walks over a non-native object
info: bugzilla.mozilla.org/show_bug.cgi?id=901380
esid: pending
---*/

// A little trickiness to account for the undefined-ness of property
// enumeration order.
var first = "unset";

var observedTypedArrayElementCount = 0;

var typedArray = null;

var obj = JSON.parse('{ "a": 0, "b": 1 }', function(prop, v) {
  if (first === "unset")
  {
    first = prop;
    var second = (prop === "a") ? "b" : "a";
    typedArray = new Int8Array(1);
    Object.defineProperty(this, second, { value: typedArray });
  }
  if (this instanceof Int8Array)
  {
    assert.sameValue(prop, "0");
    observedTypedArrayElementCount++;
  }
  return v;
});

if (first === "a")
{
  assert.sameValue(obj.a, 0);
  assert.sameValue(obj.b, typedArray);
}
else
{
  assert.sameValue(obj.a, typedArray);
  assert.sameValue(obj.b, 1);
}

assert.sameValue(observedTypedArrayElementCount, 1);
