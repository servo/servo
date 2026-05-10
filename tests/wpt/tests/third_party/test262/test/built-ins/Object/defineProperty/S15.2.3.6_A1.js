// Copyright 2011 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If a particular API exists (document.createElement, as happens to
    exist in a browser environment), check if the form objects it makes
    obey the constraints that even host objects must obey. In this
    case, that if defineProperty seems to have successfully installed a
    non-configurable getter, that it is still there.
es5id: 15.2.3.6_A1
description: Do getters on HTMLFormElements disappear?
---*/

function getter() {
  return 'gotten';
}

if (typeof document !== 'undefined' &&
  typeof document.createElement === 'function') {
  var f = document.createElement("form");
  var refused = false;
  try {
    Object.defineProperty(f, 'foo', {
      get: getter,
      set: void 0
    });
  } catch (err) {
    // A host object may refuse to install the getter
    refused = true;
  }
  if (!refused) {
    var desc = Object.getOwnPropertyDescriptor(f, 'foo');
    assert.sameValue(desc.get, getter, 'The value of desc.get is expected to equal the value of getter');
  }
}
