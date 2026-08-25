// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable string wrapper
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/
var str1 = new String("yuck\uD83D\uDCA9")
// String wrapper objects are not concat-spreadable by default
assert.compareArray([str1], [].concat(str1), '[str1] must return the same value returned by [].concat(str1)');

// String wrapper objects may be individually concat-spreadable
str1[Symbol.isConcatSpreadable] = true;
assert.compareArray(["y", "u", "c", "k", "\uD83D", "\uDCA9"], [].concat(str1),
  '["y", "u", "c", "k", "uD83D", "uDCA9"] must return the same value returned by [].concat(str1)'
);

String.prototype[Symbol.isConcatSpreadable] = true;
// String wrapper objects may be concat-spreadable
assert.compareArray(["y", "u", "c", "k", "\uD83D", "\uDCA9"], [].concat(new String("yuck\uD83D\uDCA9")),
  '["y", "u", "c", "k", "uD83D", "uDCA9"] must return the same value returned by [].concat(new String("yuckuD83DuDCA9"))'
);

// String values are never concat-spreadable
assert.compareArray(["yuck\uD83D\uDCA9"], [].concat("yuck\uD83D\uDCA9"),
  '["yuckuD83DuDCA9"] must return the same value returned by [].concat("yuckuD83DuDCA9")'
);
delete String.prototype[Symbol.isConcatSpreadable];
