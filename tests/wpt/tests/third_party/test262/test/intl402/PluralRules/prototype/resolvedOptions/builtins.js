// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.resolvedOptions
description: >
    Tests that Intl.PluralRules.prototype.resolvedOptions meets the requirements for
    built-in objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Zibi Braniecki
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.PluralRules.prototype.resolvedOptions), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(Intl.PluralRules.prototype.resolvedOptions),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.PluralRules.prototype.resolvedOptions), Function.prototype);

assert.sameValue(Intl.PluralRules.prototype.resolvedOptions.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(Intl.PluralRules.prototype.resolvedOptions), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
