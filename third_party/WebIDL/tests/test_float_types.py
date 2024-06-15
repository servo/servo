import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
        typedef float myFloat;
        typedef unrestricted float myUnrestrictedFloat;
        interface FloatTypes {
          attribute float f;
          attribute unrestricted float uf;
          attribute double d;
          attribute unrestricted double ud;
          [LenientFloat]
          attribute float lf;
          [LenientFloat]
          attribute double ld;

          undefined m1(float arg1, double arg2, float? arg3, double? arg4,
                       myFloat arg5, unrestricted float arg6,
                       unrestricted double arg7, unrestricted float? arg8,
                       unrestricted double? arg9, myUnrestrictedFloat arg10);
          [LenientFloat]
          undefined m2(float arg1, double arg2, float? arg3, double? arg4,
                       myFloat arg5, unrestricted float arg6,
                       unrestricted double arg7, unrestricted float? arg8,
                       unrestricted double? arg9, myUnrestrictedFloat arg10);
          [LenientFloat]
          undefined m3(float arg);
          [LenientFloat]
          undefined m4(double arg);
          [LenientFloat]
          undefined m5((float or FloatTypes) arg);
          [LenientFloat]
          undefined m6(sequence<float> arg);
        };
    """
    )

    results = parser.finish()

    harness.check(len(results), 3, "Should be two typedefs and one interface.")
    iface = results[2]
    harness.ok(isinstance(iface, WebIDL.IDLInterface), "Should be an IDLInterface")
    types = [a.type for a in iface.members if a.isAttr()]
    harness.ok(types[0].isFloat(), "'float' is a float")
    harness.ok(not types[0].isUnrestricted(), "'float' is not unrestricted")
    harness.ok(types[1].isFloat(), "'unrestricted float' is a float")
    harness.ok(types[1].isUnrestricted(), "'unrestricted float' is unrestricted")
    harness.ok(types[2].isFloat(), "'double' is a float")
    harness.ok(not types[2].isUnrestricted(), "'double' is not unrestricted")
    harness.ok(types[3].isFloat(), "'unrestricted double' is a float")
    harness.ok(types[3].isUnrestricted(), "'unrestricted double' is unrestricted")

    method = iface.members[6]
    harness.ok(isinstance(method, WebIDL.IDLMethod), "Should be an IDLMethod")
    argtypes = [a.type for a in method.signatures()[0][1]]
    for idx, type in enumerate(argtypes):
        harness.ok(type.isFloat(), "Type %d should be float" % idx)
        harness.check(
            type.isUnrestricted(),
            idx >= 5,
            "Type %d should %sbe unrestricted" % (idx, "" if idx >= 4 else "not "),
        )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface FloatTypes {
              [LenientFloat]
              long m(float arg);
            };
        """
        )
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "[LenientFloat] only allowed on methods returning undefined")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface FloatTypes {
              [LenientFloat]
              undefined m(unrestricted float arg);
            };
        """
        )
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "[LenientFloat] only allowed on methods with unrestricted float args"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface FloatTypes {
              [LenientFloat]
              undefined m(sequence<unrestricted float> arg);
            };
        """
        )
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "[LenientFloat] only allowed on methods with unrestricted float args (2)"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface FloatTypes {
              [LenientFloat]
              undefined m((unrestricted float or FloatTypes) arg);
            };
        """
        )
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "[LenientFloat] only allowed on methods with unrestricted float args (3)"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface FloatTypes {
              [LenientFloat]
              readonly attribute float foo;
            };
        """
        )
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "[LenientFloat] only allowed on writable attributes")
