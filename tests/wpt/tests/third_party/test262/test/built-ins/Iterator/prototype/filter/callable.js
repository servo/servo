// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Iterator.prototype.filter is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.filter.call(g(), () => false);

let iter = g();
iter.filter(() => false);
