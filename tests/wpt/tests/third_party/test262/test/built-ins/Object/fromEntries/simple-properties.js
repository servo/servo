// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Creates data properties which are enumerable, writable, and configurable.
esid: sec-object.fromentries
includes: [propertyHelper.js]
features: [Object.fromEntries]
---*/

var result = Object.fromEntries([['key', 'value']]);
verifyProperty(result, "key", {
  enumerable: true,
  writable: true,
  configurable: true,
});
