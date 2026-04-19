// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.supportedLocalesOf
description: >
    Tests that Intl.PluralRules.supportedLocalesOf meets the requirements for
    built-in objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Zibi Braniecki
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.PluralRules.supportedLocalesOf), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(Intl.PluralRules.supportedLocalesOf),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.PluralRules.supportedLocalesOf), Function.prototype);

assert.sameValue(Intl.PluralRules.supportedLocalesOf.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(Intl.PluralRules.supportedLocalesOf), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
