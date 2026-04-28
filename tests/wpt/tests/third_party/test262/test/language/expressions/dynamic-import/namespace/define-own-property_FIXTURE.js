// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

export var local1;
var local2;
export { local2 as renamed };
export { local1 as indirect } from './define-own-property_FIXTURE.js';
