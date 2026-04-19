// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2.4.2
description: Subclassed GeneratorFunction instances `name` property
info: |
  25.2.4.2 name

  The specification for the name property of Function instances given in
  19.2.4.2 also applies to GeneratorFunction instances.

  19.2.4.2 name

  The value of the name property is an String that is descriptive of the
  function. The name has no semantic significance but is typically a variable or
  property name that is used to refer to the function at its point of definition
  in ECMAScript code. This property has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.

  Anonymous functions objects that do not have a contextual name associated with
  them by this specification do not have a name own property but inherit the
  name property of %FunctionPrototype%.

  19.2.1.1.1 RuntimeSemantics: CreateDynamicFunction(constructor, newTarget,
  kind, args)

  ...
  29. Perform SetFunctionName(F, "anonymous").
  ...
includes: [propertyHelper.js]
---*/

var GeneratorFunction = Object.getPrototypeOf(function* () {}).constructor;

class GFn extends GeneratorFunction {}

var gfn = new GFn('a', 'b', 'return a + b');

verifyProperty(gfn, 'name', {
  value: 'anonymous',
  writable: false,
  enumerable: false,
  configurable: true,
});
