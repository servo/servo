// Copyright (C) 2025 Mozilla Foundation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-text-module
description: Does not parse imported module as JavaScript
info: |
  # 16.2.1.8.x CreateTextModule ( source )

  The abstract operation CreateTextModule takes argument source (a String) and
  returns a normal completion containing a Synthetic Module Record.
  It performs the following steps when called:

    1. Return CreateDefaultExportSyntheticModule(source).
flags: [module]
features: [import-attributes, import-text]
---*/

import value from './text-javascript_FIXTURE.js' with { type: 'text' };

assert.sameValue(typeof value, 'string');
