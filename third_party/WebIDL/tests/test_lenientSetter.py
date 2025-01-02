# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import WebIDL


def should_throw(parser, harness, message, code):
    parser = parser.reset()
    threw = False
    try:
        parser.parse(code)
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown: %s" % message)


def WebIDLTest(parser, harness):
    # The [LegacyLenientSetter] extended attribute MUST take no arguments.
    should_throw(
        parser,
        harness,
        "no arguments",
        """
        interface I {
          [LegacyLenientSetter=X] readonly attribute long A;
        };
    """,
    )

    # An attribute with the [LegacyLenientSetter] extended attribute MUST NOT
    # also be declared with the [PutForwards] extended attribute.
    should_throw(
        parser,
        harness,
        "PutForwards",
        """
        interface I {
          [PutForwards=B, LegacyLenientSetter] readonly attribute J A;
        };
        interface J {
          attribute long B;
        };
    """,
    )

    # An attribute with the [LegacyLenientSetter] extended attribute MUST NOT
    # also be declared with the [Replaceable] extended attribute.
    should_throw(
        parser,
        harness,
        "Replaceable",
        """
        interface I {
          [Replaceable, LegacyLenientSetter] readonly attribute J A;
        };
    """,
    )

    # The [LegacyLenientSetter] extended attribute MUST NOT be used on an
    # attribute that is not read only.
    should_throw(
        parser,
        harness,
        "writable attribute",
        """
        interface I {
          [LegacyLenientSetter] attribute long A;
        };
    """,
    )

    # The [LegacyLenientSetter] extended attribute MUST NOT be used on a
    # static attribute.
    should_throw(
        parser,
        harness,
        "static attribute",
        """
        interface I {
          [LegacyLenientSetter] static readonly attribute long A;
        };
    """,
    )
