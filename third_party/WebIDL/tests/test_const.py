import WebIDL

expected = [
    ("::TestConsts::zero", "zero", "Byte", 0),
    ("::TestConsts::b", "b", "Byte", -1),
    ("::TestConsts::o", "o", "Octet", 2),
    ("::TestConsts::s", "s", "Short", -3),
    ("::TestConsts::us", "us", "UnsignedShort", 4),
    ("::TestConsts::l", "l", "Long", -5),
    ("::TestConsts::ul", "ul", "UnsignedLong", 6),
    ("::TestConsts::ull", "ull", "UnsignedLongLong", 7),
    ("::TestConsts::ll", "ll", "LongLong", -8),
    ("::TestConsts::t", "t", "Boolean", True),
    ("::TestConsts::f", "f", "Boolean", False),
    ("::TestConsts::fl", "fl", "Float", 0.2),
    ("::TestConsts::db", "db", "Double", 0.2),
    ("::TestConsts::ufl", "ufl", "UnrestrictedFloat", 0.2),
    ("::TestConsts::udb", "udb", "UnrestrictedDouble", 0.2),
    ("::TestConsts::fli", "fli", "Float", 2),
    ("::TestConsts::dbi", "dbi", "Double", 2),
    ("::TestConsts::ufli", "ufli", "UnrestrictedFloat", 2),
    ("::TestConsts::udbi", "udbi", "UnrestrictedDouble", 2),
]


def WebIDLTest(parser, harness):
    parser.parse(
        """
        interface TestConsts {
          const byte zero = 0;
          const byte b = -1;
          const octet o = 2;
          const short s = -3;
          const unsigned short us = 0x4;
          const long l = -0X5;
          const unsigned long ul = 6;
          const unsigned long long ull = 7;
          const long long ll = -010;
          const boolean t = true;
          const boolean f = false;
          const float fl = 0.2;
          const double db = 0.2;
          const unrestricted float ufl = 0.2;
          const unrestricted double udb = 0.2;
          const float fli = 2;
          const double dbi = 2;
          const unrestricted float ufli = 2;
          const unrestricted double udbi = 2;
        };
    """
    )

    results = parser.finish()

    harness.ok(True, "TestConsts interface parsed without error.")
    harness.check(len(results), 1, "Should be one production.")
    iface = results[0]
    harness.ok(isinstance(iface, WebIDL.IDLInterface), "Should be an IDLInterface")
    harness.check(
        iface.identifier.QName(), "::TestConsts", "Interface has the right QName"
    )
    harness.check(iface.identifier.name, "TestConsts", "Interface has the right name")
    harness.check(
        len(iface.members), len(expected), "Expect %s members" % len(expected)
    )

    for const, (QName, name, type, value) in zip(iface.members, expected):
        harness.ok(isinstance(const, WebIDL.IDLConst), "Should be an IDLConst")
        harness.ok(const.isConst(), "Const is a const")
        harness.ok(not const.isAttr(), "Const is not an attr")
        harness.ok(not const.isMethod(), "Const is not a method")
        harness.check(const.identifier.QName(), QName, "Const has the right QName")
        harness.check(const.identifier.name, name, "Const has the right name")
        harness.check(str(const.type), type, "Const has the right type")
        harness.ok(const.type.isPrimitive(), "All consts should be primitive")
        harness.check(
            str(const.value.type),
            str(const.type),
            "Const's value has the same type as the type",
        )
        harness.check(const.value.value, value, "Const value has the right value.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface TestConsts {
            const boolean? zero = 0;
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Nullable types are not allowed for consts.")
