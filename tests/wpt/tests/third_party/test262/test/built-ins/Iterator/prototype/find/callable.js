// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.find
description: >
  Iterator.prototype.find is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.find.call(g(), () => {});

let iter = g();
iter.find(() => {});
