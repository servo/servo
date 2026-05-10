// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Verify property descriptor
includes: [propertyHelper.js]
---*/

var obj;
var prop = 'prop';

function reset(desc) {
  obj = {};
  Object.defineProperty(obj, prop, desc);
}

function checkDesc(desc) {
  reset(desc);
  assert(verifyProperty(obj, prop, desc));

  reset(desc);
  assert(verifyProperty(obj, prop, { enumerable: desc.enumerable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { writable: desc.writable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { configurable: desc.configurable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { configurable: desc.configurable, enumerable: desc.enumerable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { configurable: desc.configurable, writable: desc.writable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { writable: desc.writable, enumerable: desc.enumerable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { enumerable: desc.enumerable, configurable: desc.configurable }));
}

checkDesc({ enumerable: true, configurable: true, writable: true });
checkDesc({ enumerable: false, writable: false, configurable: false });
checkDesc({ enumerable: true, writable: false, configurable: false });
checkDesc({ enumerable: false, writable: true, configurable: false });
checkDesc({ enumerable: false, writable: false, configurable: true });
checkDesc({ enumerable: true, writable: false, configurable: true });
checkDesc({ enumerable: true, writable: true, configurable: false });
checkDesc({ enumerable: false, writable: true, configurable: true });
