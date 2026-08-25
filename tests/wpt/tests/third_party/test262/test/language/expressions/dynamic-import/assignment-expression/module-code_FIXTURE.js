// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// exports: default === 42, local1 === 'Test262', renamed === 'TC39', indirect === 'Test262'

export var local1 = 'Test262';
var local2 = 'TC39';
export { local2 as renamed };
export { local1 as indirect } from './module-code_FIXTURE.js';
export default 42;
