// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reverse
description: Array.prototype.reverse should not iterate items if there is only one entry
info: |
  Array.prototype.reverse ( )

  Let O be ? ToObject(this value).
  Let len be ? LengthOfArrayLike(O).
  Let middle be floor(len / 2).
  Let lower be 0.
  Repeat, while lower ≠ middle,
    ...
  Return O.
---*/

let a = [1];

Object.freeze(a);

a.reverse();
