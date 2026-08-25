// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-parse-json-module
description: The same object representation is returned to all import sites
flags: [module, async]
features: [import-attributes, json-modules, globalThis, dynamic-import]
---*/

import viaStaticImport1 from './json-idempotency_FIXTURE.json' with { type: 'json' };
import {default as viaStaticImport2} from './json-idempotency_FIXTURE.json' with { type: 'json' };
import './json-idempotency-indirect_FIXTURE.js';

assert.sameValue(viaStaticImport1, viaStaticImport2);
assert.sameValue(globalThis.viaSecondModule, viaStaticImport1);

import('./json-idempotency_FIXTURE.json', { with: { type: 'json' } })
  .then(function(viaDynamicImport) {
    assert.sameValue(viaDynamicImport.default, viaStaticImport1);
  })
  .then($DONE, $DONE);
