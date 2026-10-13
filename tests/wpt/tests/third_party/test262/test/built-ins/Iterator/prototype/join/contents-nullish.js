// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join formats nullish iterator contents as an empty string.
features: [Iterator.prototype.join]
---*/

assert.sameValue(
  ['one', null, 'two', undefined].values().join(),
  'one,,two,'
);

assert.sameValue(
  ['one', null, 'two', undefined, 'three'].values().join(),
  'one,,two,,three'
);
