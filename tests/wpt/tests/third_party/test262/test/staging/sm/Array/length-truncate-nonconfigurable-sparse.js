/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Array length redefinition behavior with non-configurable elements
info: bugzilla.mozilla.org/show_bug.cgi?id=858381
esid: pending
---*/

function addDataProperty(obj, prop, value, enumerable, configurable, writable)
{
  var desc =
    { enumerable: enumerable,
      configurable: configurable,
      writable: writable,
      value: value };
  Object.defineProperty(obj, prop, desc);
}

function nonstrict()
{
  var arr = [0, , 2, , , 5];

  addDataProperty(arr,  31415926, "foo", true,  true,  true);
  addDataProperty(arr, 123456789, "bar", true,  true,  false);
  addDataProperty(arr,   8675309, "qux", false, true,  true);
  addDataProperty(arr,   1735039, "eit", false, true,  false);
  addDataProperty(arr, 987654321, "fun", false, true,  false);

  // non-array indexes to spice things up
  addDataProperty(arr, "foopy", "sdfsd", false, false, false);
  addDataProperty(arr, 4294967296, "psych", true, false, false);
  addDataProperty(arr, 4294967295, "psych", true, false, false);

  addDataProperty(arr,  27182818, "eep", false, false, false);

  // Truncate...but only as far as possible.
  arr.length = 1;

  assert.sameValue(arr.length, 27182819);

  var props = Object.getOwnPropertyNames(arr).sort();
  var expected =
    ["0", "2", "5", "1735039", "8675309", "27182818",
     "foopy", "4294967296", "4294967295", "length"].sort();

  assert.sameValue(props.length, expected.length);
  for (var i = 0; i < props.length; i++)
    assert.sameValue(props[i], expected[i], "unexpected property: " + props[i]);
}
nonstrict();

function strict()
{
  "use strict";

  var arr = [0, , 2, , , 5];

  addDataProperty(arr,  31415926, "foo", true,  true,  true);
  addDataProperty(arr, 123456789, "bar", true,  true,  false);
  addDataProperty(arr,   8675309, "qux", false, true,  true);
  addDataProperty(arr,   1735039, "eit", false, true,  false);
  addDataProperty(arr, 987654321, "fun", false, true,  false);

  // non-array indexes to spice things up
  addDataProperty(arr, "foopy", "sdfsd", false, false, false);
  addDataProperty(arr, 4294967296, "psych", true, false, false);
  addDataProperty(arr, 4294967295, "psych", true, false, false);

  addDataProperty(arr,  27182818, "eep", false, false, false);

  assert.throws(TypeError, function() {
    arr.length = 1;
  }, "non-configurable property should trigger TypeError");

  assert.sameValue(arr.length, 27182819);

  var props = Object.getOwnPropertyNames(arr).sort();
  var expected =
    ["0", "2", "5", "1735039", "8675309", "27182818",
     "foopy", "4294967296", "4294967295", "length"].sort();

  assert.sameValue(props.length, expected.length);
  for (var i = 0; i < props.length; i++)
    assert.sameValue(props[i], expected[i], "unexpected property: " + props[i]);
}
strict();
