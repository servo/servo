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
      if (count > 2)
        return 0;
      throw new SyntaxError("fnord");
    }
  };

var arr = [];
Object.defineProperty(arr, "length", { value: 0, writable: false });

assert.throws(SyntaxError, function() {
  Object.defineProperty(arr, "length",
                        {
                          value: convertible,
                          writable: true,
                          configurable: true,
                          enumerable: true
                        });
});

assert.sameValue(count, 1);
assert.sameValue(arr.length, 0);
