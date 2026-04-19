// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.3.7
description: Template objects are frozen (as demonstrated outside of strict mode)
info: |
    The first argument to a tagged template should be frozen and define a `raw`
    property that is also frozen.
flags: [noStrict]
---*/

var templateObject = null;
var threwError = false;
(function(parameter) {
  templateObject = parameter;
})``;

assert(templateObject !== null);
templateObject.test262Prop = true;

assert.sameValue(
  templateObject.test262Prop, undefined, 'The template object is frozen'
);

templateObject.raw.test262Prop = true;

assert.sameValue(
  templateObject.raw.test262Prop, undefined, 'The "raw" object is frozen'
);
assert(
  templateObject.raw !== undefined , 'Template object defines a `raw` property'
);
