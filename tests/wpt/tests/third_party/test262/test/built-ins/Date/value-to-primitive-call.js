// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: Invocation of `Symbol.toPrimitive` method of object value
info: |
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot, then
        i. Let tv be thisTimeValue(value).
     b. Else,
        i. Let v be ? ToPrimitive(value).
        [...]


  ToPrimitive ( input [ , PreferredType ] )

  1. If PreferredType was not passed, let hint be "default".
  2. Else if PreferredType is hint String, let hint be "string".
  3. Else PreferredType is hint Number, let hint be "number".
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
  5. If exoticToPrim is not undefined, then
     a. Let result be ? Call(exoticToPrim, input, « hint »).
     [...]
features: [Symbol.toPrimitive]
---*/

var spyToPrimitive = {};
var callCount = 0;
var thisValue, args;
spyToPrimitive[Symbol.toPrimitive] = function() {
  thisValue = this;
  args = arguments;
  callCount += 1;
};

new Date(spyToPrimitive);

assert.sameValue(callCount, 1, 'function invoked exactly once');
assert.sameValue(thisValue, spyToPrimitive, 'function "this" value');
assert.sameValue(args.length, 1, 'function invoked with exactly one argument');
assert.sameValue(args[0], 'default', 'argument value');
