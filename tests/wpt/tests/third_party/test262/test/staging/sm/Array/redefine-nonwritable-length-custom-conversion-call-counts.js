/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Assertion redefining non-writable length to a non-numeric value
info: bugzilla.mozilla.org/show_bug.cgi?id=866700
esid: pending
---*/

var count = 0;

var convertible =
  {
    valueOf: function()
    {
      count++;
      return 0;
    }
  };

var arr = [];
Object.defineProperty(arr, "length", { value: 0, writable: false });

Object.defineProperty(arr, "length", { value: convertible });
assert.sameValue(count, 2);

Object.defineProperty(arr, "length", { value: convertible });
assert.sameValue(count, 4);

assert.sameValue(arr.length, 0);
