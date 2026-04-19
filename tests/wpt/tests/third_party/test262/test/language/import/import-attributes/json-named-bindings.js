// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-parse-json-module
description: Does not define named bindings
info: |
  In the early design of JSON modules, contributors considered allowing the
  properties of object values in JSON modules to be imported directly by name.
  This was ultimately rejected, so attempting to import in this way should
  produce a SyntaxError.
flags: [module]
features: [import-attributes, json-modules]
negative:
  phase: resolution
  type: SyntaxError
---*/

$DONOTEVALUATE();

import {name} from './json-named-bindings_FIXTURE.json' with { type: 'json' };
