// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Array.prototype.reverse only gets present properties - delete property with getter
info: |
  22.1.3.20 Array.prototype.reverse ( )

  ...
  7.
    d. Let lowerExists be HasProperty(O, lowerP).
    e. ReturnIfAbrupt(lowerExists).
    f. If lowerExists is true, then
      i.  Let lowerValue be Get(O, lowerP).
      ii. ReturnIfAbrupt(lowerValue).
    g. Let upperExists be HasProperty(O, upperP).
    h. ReturnIfAbrupt(upperExists).
    i. If upperExists is true, then
      i.  Let upperValue be Get(O, upperP).
      ii. ReturnIfAbrupt(upperValue).
esid: sec-array.prototype.reverse
---*/

var array = ["first", "second"];

Object.defineProperty(array, 0, {
  get: function() {
    array.length = 0;
    return "first";
  }
});

array.reverse();

assert.sameValue((0 in array), false, "Indexed property '0' not present");
assert.sameValue((1 in array), true, "Indexed property '1' present");
assert.sameValue(array[1], "first", "Indexed property '1' value correct");
