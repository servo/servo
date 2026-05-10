// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.every.call(g(), () => {});

let iter = g();
iter.every(() => {});
