// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-call-thisargument-argumentslist
description: The "this" value is wrapped in an object using the callee realm
info: |
  [...]
  6. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
  [...]

  9.2.1.2OrdinaryCallBindThis ( F, calleeContext, thisArgument )#

  [...]
  5. If thisMode is strict, let thisValue be thisArgument.
  6. Else,
     a. If thisArgument is null or undefined, then
        [...]
     b. Else,
        i. Let thisValue be ! ToObject(thisArgument).
        ii. NOTE ToObject produces wrapper objects using calleeRealm.
  [...]
features: [cross-realm]
---*/

var other = $262.createRealm().global;
var func = new other.Function('return this;');
var subject;

subject = func.call(true);
assert.sameValue(subject.constructor, other.Boolean, 'boolean constructor');
assert(subject instanceof other.Boolean, 'boolean instanceof');

subject = func.call(1);
assert.sameValue(subject.constructor, other.Number, 'number constructor');
assert(subject instanceof other.Number, 'number instanceof');

subject = func.call('');
assert.sameValue(subject.constructor, other.String, 'string constructor');
assert(subject instanceof other.String, 'string instanceof');

subject = func.call({});
assert.sameValue(subject.constructor, Object, 'object constructor');
assert(subject instanceof Object, 'object instanceof');
