// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  The internal prototype of Intl.DisplayNames
features: [Intl.DisplayNames]
---*/

var proto = Object.getPrototypeOf(Intl.DisplayNames);

assert.sameValue(proto, Function.prototype);
