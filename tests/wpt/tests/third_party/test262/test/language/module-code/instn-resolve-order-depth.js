// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Module dependencies are resolved following a depth-first strategy
esid: sec-moduledeclarationinstantiation
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

import './instn-resolve-order-depth-child_FIXTURE.js';
import './instn-resolve-order-depth-syntax_FIXTURE.js';
