// Copyright (C) 2020 Igalia S.L, Toru Nagashima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    BigInt in LiteralPropertyName must be valid and the property name must be
    the string representation of the numeric value.
esid: prod-PropertyName
info: |
    PropertyName[Yield, Await]:
        LiteralPropertyName
        ComputedPropertyName[?Yield, ?Await]

    LiteralPropertyName:
        IdentifierName
        StringLiteral
        NumericLiteral

    NumericLiteral:
        DecimalLiteral
        DecimalBigIntegerLiteral

    LiteralPropertyName: NumericLiteral
        1. Let _nbr_ be the NumericValue of |NumericLiteral|.
        1. Return ! ToString(_nbr_).
features: [BigInt, class, destructuring-binding, let]
---*/

// Property

let o = { 999999999999999999n: true }; // greater than max safe integer

assert.sameValue(o["999999999999999999"], true,
    "the property name must be the string representation of the numeric value.");

// MethodDeclaration

o = { 1n() { return "bar"; } };
assert.sameValue(o["1"](), "bar",
    "the property name must be the string representation of the numeric value.");

class C {
  1n() { return "baz"; }
}

let c = new C();
assert.sameValue(c["1"](), "baz",
    "the property name must be the string representation of the numeric value.");

// Destructuring

let { 1n: a } = { "1": "foo" };
assert.sameValue(a, "foo",
    "the property name must be the string representation of the numeric value.");
