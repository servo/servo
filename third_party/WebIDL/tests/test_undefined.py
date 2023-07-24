import WebIDL


def WebIDLTest(parser, harness):
    try:
        parser.parse(
            """
            dictionary Dict {
              undefined undefinedMember;
              double bar;
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "undefined must not be used as the type of a dictionary member")

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            dictionary Dict {
              (undefined or double) undefinedMemberOfUnionInDict;
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of a dictionary member, "
        "whether directly or in a union",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              double bar(undefined foo);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a regular operation)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              getter double(undefined name);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a getter)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              setter undefined(DOMString name, undefined value);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a setter)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              deleter undefined (undefined name);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a deleter)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              constructor (undefined foo);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a constructor)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            callback Callback = undefined (undefined foo);
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a callback)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              async iterable(undefined name);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of an async iterable "
        "iterator)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              static double bar(undefined foo);
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined must not be used as the type of an argument in any "
        "circumstance (so not as the argument of a static operation)",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              const undefined FOO = undefined;
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined is not a valid type for a constant",
    )

    parser = parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface Foo {
              const any FOO = undefined;
            };
            """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "undefined is not a valid value for a constant",
    )
