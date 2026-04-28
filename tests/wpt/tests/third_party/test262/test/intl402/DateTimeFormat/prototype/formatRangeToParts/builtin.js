// Copyright 2012 Mozilla Corporation. All rights reserved.
// Copyright 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
    Tests that the Intl.DateTimeFormat.prototype.formatRangeToParts function meets the
    requirements for built-in objects defined by the ECMAScript Language
    Specification.
includes: [isConstructor.js]
features: [Reflect.construct,Intl.DateTimeFormat-formatRange]
---*/

const formatRangeToParts = Intl.DateTimeFormat.prototype.formatRangeToParts;

assert.sameValue(Object.prototype.toString.call(formatRangeToParts), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(formatRangeToParts),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(formatRangeToParts), Function.prototype);

assert.sameValue(formatRangeToParts.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(formatRangeToParts), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
