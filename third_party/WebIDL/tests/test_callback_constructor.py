import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
        interface TestCallbackConstructor {
          attribute CallbackConstructorType? constructorAttribute;
        };

        callback constructor CallbackConstructorType = TestCallbackConstructor (unsigned long arg);
    """
    )

    results = parser.finish()

    harness.ok(True, "TestCallbackConstructor interface parsed without error.")
    harness.check(len(results), 2, "Should be two productions.")
    iface = results[0]
    harness.ok(isinstance(iface, WebIDL.IDLInterface), "Should be an IDLInterface")
    harness.check(
        iface.identifier.QName(),
        "::TestCallbackConstructor",
        "Interface has the right QName",
    )
    harness.check(
        iface.identifier.name, "TestCallbackConstructor", "Interface has the right name"
    )
    harness.check(len(iface.members), 1, "Expect %s members" % 1)

    attr = iface.members[0]
    harness.ok(isinstance(attr, WebIDL.IDLAttribute), "Should be an IDLAttribute")
    harness.ok(attr.isAttr(), "Should be an attribute")
    harness.ok(not attr.isMethod(), "Attr is not an method")
    harness.ok(not attr.isConst(), "Attr is not a const")
    harness.check(
        attr.identifier.QName(),
        "::TestCallbackConstructor::constructorAttribute",
        "Attr has the right QName",
    )
    harness.check(
        attr.identifier.name, "constructorAttribute", "Attr has the right name"
    )
    t = attr.type
    harness.ok(not isinstance(t, WebIDL.IDLWrapperType), "Attr has the right type")
    harness.ok(isinstance(t, WebIDL.IDLNullableType), "Attr has the right type")
    harness.ok(t.isCallback(), "Attr has the right type")

    callback = results[1]
    harness.ok(callback.isConstructor(), "Callback is constructor")

    parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [LegacyTreatNonObjectAsNull]
            callback constructor CallbackConstructorType = object ();
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should throw on LegacyTreatNonObjectAsNull callback constructors"
    )

    parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [MOZ_CAN_RUN_SCRIPT_BOUNDARY]
            callback constructor CallbackConstructorType = object ();
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should not permit MOZ_CAN_RUN_SCRIPT_BOUNDARY callback constructors"
    )
