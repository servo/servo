# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

def should_throw(parser, harness, message, code):
    parser = parser.reset();
    threw = False
    try:
        parser.parse(code)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown: %s" % message)


def WebIDLTest(parser, harness):
    # The [LenientSetter] extended attribute MUST take no arguments.
    should_throw(parser, harness, "no arguments", """
        interface I {
          [LenientSetter=X] readonly attribute long A;
        };
    """)

    # An attribute with the [LenientSetter] extended attribute MUST NOT
    # also be declared with the [PutForwards] extended attribute.
    should_throw(parser, harness, "PutForwards", """
        interface I {
          [PutForwards=B, LenientSetter] readonly attribute J A;
        };
        interface J {
          attribute long B;
        };
    """)

    # An attribute with the [LenientSetter] extended attribute MUST NOT
    # also be declared with the [Replaceable] extended attribute.
    should_throw(parser, harness, "Replaceable", """
        interface I {
          [Replaceable, LenientSetter] readonly attribute J A;
        };
    """)

    # The [LenientSetter] extended attribute MUST NOT be used on an
    # attribute that is not read only.
    should_throw(parser, harness, "writable attribute", """
        interface I {
          [LenientSetter] attribute long A;
        };
    """)

    # The [LenientSetter] extended attribute MUST NOT be used on a
    # static attribute.
    should_throw(parser, harness, "static attribute", """
        interface I {
          [LenientSetter] static readonly attribute long A;
        };
    """)
