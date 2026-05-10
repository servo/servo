// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

import value from './json-idempotency_FIXTURE.json' with { type: 'json' };

globalThis.viaSecondModule = value;
