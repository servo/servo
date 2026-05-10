// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Iterator.prototype.take is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.take.call(g(), 0);

let iter = g();
iter.take(0);
