// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-constructor-prototype
description: AsyncFunction has a prototype property with writable false, enumerable false, configurable false.
includes: [propertyHelper.js]
---*/

var AsyncFunction = async function foo() {}.constructor;
verifyNotConfigurable(AsyncFunction, 'prototype');
verifyNotWritable(AsyncFunction, 'prototype');
verifyNotEnumerable(AsyncFunction, 'prototype');
