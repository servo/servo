import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers1 {
              const byte thing1 = 1;
              readonly attribute long thing1;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers1.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers2 {
              readonly attribute long thing1;
              const byte thing1 = 1;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers2.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers3 {
              getter boolean thing1(DOMString name);
              readonly attribute long thing1;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers3.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers4 {
              const byte thing1 = 1;
              long thing1();
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers4.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers5 {
              static long thing1();
              undefined thing1();
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        not threw, "Should not have thrown for IdentifierConflictAcrossMembers5."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin IdentifierConflictAcrossMembers6Mixin {
              undefined thing1();
            };
            interface IdentifierConflictAcrossMembers6 {
              static long thing1();
            };
            IdentifierConflictAcrossMembers6 includes IdentifierConflictAcrossMembers6Mixin;
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        not threw, "Should not have thrown for IdentifierConflictAcrossMembers6."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers7 {
              const byte thing1 = 1;
              static readonly attribute long thing1;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers7.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers8 {
              readonly attribute long thing1 = 1;
              static readonly attribute long thing1;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers8.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface IdentifierConflictAcrossMembers9 {
              void thing1();
              static readonly attribute long thing1;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for IdentifierConflictAcrossMembers9.")
