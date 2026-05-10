// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-__proto__-property-names-in-object-initializers
es6id: B.3.1
description: >
    The syntax error for duplicate `__proto__` property is not valid if the duplicate is a
    ComputedPropertyName
info: |
    B.3.1__proto__ Property Names in Object Initializers

    It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains any duplicate
    entries for  "__proto__" and at least two of those entries were obtained from productions of
    the form
    PropertyDefinition : PropertyName : AssignmentExpression .

    12.2.6.6 Static Semantics: PropertyNameList

    ...
    3. Append PropName of PropertyDefinition to the end of list.
    ...

    12.2.6.5 Static Semantics: PropName

    ComputedPropertyName : [ AssignmentExpression ]
        1. Return empty.
---*/

var obj;
var proto = {};
var ownProp = {};

obj = {
    __proto__: proto,
    ['__proto__']: {},
    ['__proto__']: ownProp
};

assert.sameValue(
    Object.getPrototypeOf(obj),
    proto,
    'prototype is defined'
);

assert(
    Object.prototype.hasOwnProperty.call(obj, '__proto__'),
    'has own property __proto__'
);

assert.sameValue(
    obj.__proto__,
    ownProp,
    'own property value'
);
