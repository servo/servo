// Copyright 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
info: Object.hasOwn has not prototype property
description: >
    Checking if obtaining the prototype property of Object.hasOwn fails
author: Jamie Kyle
features: [Object.hasOwn]
---*/

assert.sameValue(Object.hasOwn.prototype, undefined);
