// Copyright (C) 2025 Mozilla Foundation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-text-module
description: Dynamic import of text files
flags: [async]
features: [dynamic-import, import-attributes, import-text]
---*/

import('./2nd-param_FIXTURE.json', { with: { type: 'text' } })
  .then((module) => {
    assert.sameValue(typeof module.default, 'string');
  })
  .then($DONE, $DONE);
