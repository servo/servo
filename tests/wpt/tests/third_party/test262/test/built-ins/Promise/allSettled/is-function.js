// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: Promise.allSettled is callable
features: [Promise.allSettled]
---*/

assert.sameValue(typeof Promise.allSettled, 'function');
