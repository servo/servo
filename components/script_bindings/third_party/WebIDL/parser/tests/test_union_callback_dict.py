import WebIDL


def WebIDLTest(parser, harness):
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary TestDict {
            DOMString member;
            };
            [LegacyTreatNonObjectAsNull] callback TestCallback = undefined ();
            typedef (TestCallback or TestDict) TestUnionCallbackDict;
            """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow Dict/Callback union where callback is [LegacyTreatNonObjectAsNull]",
    )

    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            dictionary TestDict {
            DOMString member;
            };
            [LegacyTreatNonObjectAsNull] callback TestCallback = undefined ();
            typedef (TestDict or TestCallback) TestUnionCallbackDict;
            """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow Dict/Callback union where callback is [LegacyTreatNonObjectAsNull]",
    )

    parser = parser.reset()

    parser.parse(
        """
        dictionary TestDict {
          DOMString member;
        };
        callback TestCallback = undefined ();
        typedef (TestCallback or TestDict) TestUnionCallbackDict;
        """
    )
    results = parser.finish()

    harness.ok(True, "TestUnionCallbackDict interface parsed without error")
    harness.check(len(results), 3, "Document should have 3 types")

    myDict = results[0]
    harness.ok(isinstance(myDict, WebIDL.IDLDictionary), "Expect an IDLDictionary")

    myCallback = results[1]
    harness.ok(isinstance(myCallback, WebIDL.IDLCallback), "Expect an IDLCallback")

    myUnion = results[2]
    harness.ok(isinstance(myUnion, WebIDL.IDLTypedef), "Expect a IDLTypedef")
    harness.ok(
        isinstance(myUnion.innerType, WebIDL.IDLUnionType), "Expect a IDLUnionType"
    )
    harness.ok(
        isinstance(myUnion.innerType.memberTypes[0], WebIDL.IDLCallbackType),
        "Expect a IDLCallbackType",
    )
    harness.ok(
        isinstance(myUnion.innerType.memberTypes[1], WebIDL.IDLWrapperType),
        "Expect a IDLDictionary",
    )
    harness.ok(
        (myUnion.innerType.memberTypes[0].callback == myCallback),
        "Expect left Union member to be MyCallback",
    )
    harness.ok(
        (myUnion.innerType.memberTypes[1].inner == myDict),
        "Expect right Union member to be MyDict",
    )

    parser = parser.reset()

    parser.parse(
        """
        dictionary TestDict {
          DOMString member;
        };
        callback TestCallback = undefined ();
        typedef (TestDict or TestCallback) TestUnionCallbackDict;
        """
    )
    results = parser.finish()

    harness.ok(True, "TestUnionCallbackDict interface parsed without error")
    harness.check(len(results), 3, "Document should have 3 types")

    myDict = results[0]
    harness.ok(isinstance(myDict, WebIDL.IDLDictionary), "Expect an IDLDictionary")

    myCallback = results[1]
    harness.ok(isinstance(myCallback, WebIDL.IDLCallback), "Expect an IDLCallback")

    myUnion = results[2]
    harness.ok(isinstance(myUnion, WebIDL.IDLTypedef), "Expect a IDLTypedef")
    harness.ok(
        isinstance(myUnion.innerType, WebIDL.IDLUnionType), "Expect a IDLUnionType"
    )
    harness.ok(
        isinstance(myUnion.innerType.memberTypes[0], WebIDL.IDLWrapperType),
        "Expect a IDLDictionary",
    )
    harness.ok(
        isinstance(myUnion.innerType.memberTypes[1], WebIDL.IDLCallbackType),
        "Expect a IDLCallbackType",
    )
    harness.ok(
        (myUnion.innerType.memberTypes[0].inner == myDict),
        "Expect right Union member to be MyDict",
    )
    harness.ok(
        (myUnion.innerType.memberTypes[1].callback == myCallback),
        "Expect left Union member to be MyCallback",
    )
