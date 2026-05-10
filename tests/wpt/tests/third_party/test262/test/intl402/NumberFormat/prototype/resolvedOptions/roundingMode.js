// Copyright 2021 the V8 project authors. All rights reserved.
// Copyright 2021 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.resolvedoptions
description: roundingMode property for the object returned by resolvedOptions()
features: [Intl.NumberFormat-v3]
---*/

var options;

options = new Intl.NumberFormat([], {}).resolvedOptions();
assert.sameValue(options.roundingMode, 'halfExpand', 'default');

options = new Intl.NumberFormat([], {roundingMode: 'ceil'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'ceil');

options = new Intl.NumberFormat([], {roundingMode: 'floor'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'floor');

options = new Intl.NumberFormat([], {roundingMode: 'expand'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'expand');

options = new Intl.NumberFormat([], {roundingMode: 'trunc'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'trunc');

options = new Intl.NumberFormat([], {roundingMode: 'halfCeil'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'halfCeil');

options = new Intl.NumberFormat([], {roundingMode: 'halfFloor'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'halfFloor');

options = new Intl.NumberFormat([], {roundingMode: 'halfExpand'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'halfExpand');

options = new Intl.NumberFormat([], {roundingMode: 'halfTrunc'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'halfTrunc');

options = new Intl.NumberFormat([], {roundingMode: 'halfEven'}).resolvedOptions();
assert.sameValue(options.roundingMode, 'halfEven');
