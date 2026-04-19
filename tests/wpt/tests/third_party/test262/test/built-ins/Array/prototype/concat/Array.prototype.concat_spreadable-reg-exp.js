// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat Symbol.isConcatSpreadable reg exp
includes: [compareArray.js]
features: [Symbol.isConcatSpreadable]
---*/
var re = /abc/;
// RegExps are not concat-spreadable by default
assert.compareArray([].concat(re), [re], '[].concat(/abc/) must return [re]');

// RegExps may be individually concat-spreadable
re[Symbol.isConcatSpreadable] = true;
re[0] = 1, re[1] = 2, re[2] = 3, re.length = 3;
assert.compareArray([].concat(re), [1, 2, 3], '[].concat(/abc/) must return [1, 2, 3]');

// RegExps may be concat-spreadable
RegExp.prototype[Symbol.isConcatSpreadable] = true;
RegExp.prototype.length = 3;

assert.compareArray([].concat(/abc/), [void 0, void 0, void 0],
  '[].concat(/abc/) must return [void 0, void 0, void 0]'
);
RegExp.prototype[0] = 1;
RegExp.prototype[1] = 2;
RegExp.prototype[2] = 3;
assert.compareArray([].concat(/abc/), [1, 2, 3],
  '[].concat(/abc/) must return [1, 2, 3]'
);

delete RegExp.prototype[Symbol.isConcatSpreadable];
delete RegExp.prototype[0];
delete RegExp.prototype[1];
delete RegExp.prototype[2];
delete RegExp.prototype.length;
