// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// This module is visited two times:
// First when resolving the "x" binding and then another time to resolve the
// "y" binding.
export { y as x } from './instn-star-star-cycle-2_FIXTURE.js';
export var y = 45;
