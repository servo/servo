import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
        dictionary Dict {};
        interface RecordArg {
          undefined foo(record<DOMString, Dict> arg);
        };
    """
    )

    results = parser.finish()

    harness.check(len(results), 2, "Should know about two things")
    harness.ok(
        isinstance(results[1], WebIDL.IDLInterface), "Should have an interface here"
    )
    members = results[1].members
    harness.check(len(members), 1, "Should have one member")
    harness.ok(members[0].isMethod(), "Should have method")
    signature = members[0].signatures()[0]
    args = signature[1]
    harness.check(len(args), 1, "Should have one arg")
    harness.ok(args[0].type.isRecord(), "Should have a record type here")
    harness.ok(args[0].type.inner.isDictionary(), "Should have a dictionary inner type")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface RecordUndefinedArg {
              undefined foo(record<DOMString, undefined> arg);
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "Should have thrown because record can't have undefined as value type."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Dict {
              record<DOMString, Dict> val;
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should have thrown on dictionary containing itself via record.")
