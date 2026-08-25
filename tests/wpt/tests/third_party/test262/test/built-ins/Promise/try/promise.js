// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.try return value is a Promise
features: [promise-try]
---*/

var instance = Promise.try(function () {});

assert.sameValue(instance.constructor, Promise);
assert.sameValue(instance instanceof Promise, true);
