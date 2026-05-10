// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable sparse object
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/
var obj = {
  length: 5
};
obj[Symbol.isConcatSpreadable] = true;
assert.compareArray([void 0, void 0, void 0, void 0, void 0], [].concat(obj),
  '[void 0, void 0, void 0, void 0, void 0] must return the same value returned by [].concat(obj)'
);

obj.length = 4000;
assert.compareArray(new Array(4000), [].concat(obj),
  'new Array(4000) must return the same value returned by [].concat(obj)'
);
