// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// exports: default === 1612, local1 === 'one six one two', renamed === 'star', indirect === 'one six one two'

export var local1 = 'one six one two';
var local2 = 'star';
export { local2 as renamed };
export { local1 as indirect } from './module-code-other_FIXTURE.js';
export default 1612;
