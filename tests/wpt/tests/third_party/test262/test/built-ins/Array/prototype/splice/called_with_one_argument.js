// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Array.prototype.splice deletes length-start elements when called with one argument
info: |
  22.1.3.25 Array.prototype.splice (start, deleteCount , ...items )

  ...
  9. Else if the number of actual arguments is 1, then
    a. Let insertCount be 0.
    b. Let actualDeleteCount be len – actualStart.
esid: sec-array.prototype.splice
---*/

var array = ["first", "second", "third"];

var result = array.splice(1);

assert.sameValue(array.length, 1, "array length updated");
assert.sameValue(array[0], "first", "array[0] unchanged");

assert.sameValue(result.length, 2, "result array length correct");
assert.sameValue(result[0], "second", "result[0] correct");
assert.sameValue(result[1], "third", "result[1] correct");
