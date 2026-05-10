// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: Array.prototype.toSpliced deletes the elements after start when called with one argument
info: |
  22.1.3.25 Array.prototype.toSpliced (start, deleteCount , ...items )

  ...
  9. Else if deleteCount is not present, then
    a. Let actualDeleteCount be len - actualStart.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var result = ["first", "second", "third"].toSpliced(1);

assert.compareArray(result, ["first"]);
