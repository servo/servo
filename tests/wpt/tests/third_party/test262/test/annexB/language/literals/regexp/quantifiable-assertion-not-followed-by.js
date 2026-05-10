// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
es6id: B.1.4
description: Quantifiable assertions `?!` ("not followed by")
info: |
    Term[U] ::
        [~U] QuantifiableAssertion Quantifier

    QuantifiableAssertion::
        ( ?= Disjunction )
        ( ?! Disjunction )

    The production Term::QuantifiableAssertionQuantifier evaluates the same as
    the production Term::AtomQuantifier but with QuantifiableAssertion
    substituted for Atom.

    The production Assertion::QuantifiableAssertion evaluates by evaluating
    QuantifiableAssertion to obtain a Matcher and returning that Matcher.

    Assertion (21.2.2.6) evaluation rules for the Assertion::(?=Disjunction)
    and Assertion::(?!Disjunction) productions are also used for the
    QuantifiableAssertion productions, but with QuantifiableAssertion
    substituted for Assertion.
---*/

var match;

match = /[a-e](?!Z)*/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'a', 'quantifier: *');

match = /[a-e](?!Z)+/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'e', 'quantifier: +');

match = /[a-e](?!Z)?/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'a', 'quantifier: ?');

match = /[a-e](?!Z){2}/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'e', 'quantifier: { DecimalDigits }');

match = /[a-e](?!Z){2,}/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'e', 'quantifier: { DecimalDigits , }');

match = /[a-e](?!Z){2,3}/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(
  match[0], 'e', 'quantifier: { DecimalDigits , DecimalDigits }'
);

match = /[a-e](?!Z)*?/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'a', 'quantifier: * ?');

match = /[a-e](?!Z)+?/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'e', 'quantifier: + ?');

match = /[a-e](?!Z)??/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'a', 'quantifier: ? ?');

match = /[a-e](?!Z){2}?/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'e', 'quantifier: { DecimalDigits } ?');

match = /[a-e](?!Z){2,}?/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(match[0], 'e', 'quantifier: { DecimalDigits , } ?');

match = /[a-e](?!Z){2,3}?/.exec('aZZZZ bZZZ cZZ dZ e');
assert.sameValue(
  match[0], 'e', 'quantifier: { DecimalDigits , DecimalDigits } ?'
);
