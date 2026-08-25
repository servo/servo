// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Verify property descriptor
includes: [propertyHelper.js]
---*/

var obj;
var prop = 'prop';

function reset(desc) {
  desc.value = prop;
  obj = Object.defineProperty({}, prop, desc);
}

function checkDesc(desc) {
  reset(desc);
  assert(verifyProperty(obj, prop, desc));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', enumerable: desc.enumerable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', writable: desc.writable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', configurable: desc.configurable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', configurable: desc.configurable, enumerable: desc.enumerable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', configurable: desc.configurable, writable: desc.writable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', writable: desc.writable, enumerable: desc.enumerable }));

  reset(desc);
  assert(verifyProperty(obj, prop, { value: 'prop', enumerable: desc.enumerable, configurable: desc.configurable }));
}

checkDesc({ enumerable: true, configurable: true, writable: true });
checkDesc({ enumerable: false, writable: false, configurable: false });
checkDesc({ enumerable: true, writable: false, configurable: false });
checkDesc({ enumerable: false, writable: true, configurable: false });
checkDesc({ enumerable: false, writable: false, configurable: true });
checkDesc({ enumerable: true, writable: false, configurable: true });
checkDesc({ enumerable: true, writable: true, configurable: false });
checkDesc({ enumerable: false, writable: true, configurable: true });
