import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
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
          const boolean? n = null;
          const boolean? nt = true;
          const boolean? nf = false;
        };
    """)

    results = parser.finish()

    harness.ok(True, "TestConsts interface parsed without error.")
    harness.check(len(results), 1, "Should be one production.")
    iface = results[0]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.check(iface.identifier.QName(), "::TestConsts", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestConsts", "Interface has the right name")
    harness.check(len(iface.members), 14, "Expect 14 members")

    consts = iface.members

    def checkConst(const, QName, name, type, value):
        harness.ok(isinstance(const, WebIDL.IDLConst),
                   "Should be an IDLConst")
        harness.ok(const.isConst(), "Const is a const")
        harness.ok(not const.isAttr(), "Const is not an attr")
        harness.ok(not const.isMethod(), "Const is not a method")
        harness.check(const.identifier.QName(), QName, "Const has the right QName")
        harness.check(const.identifier.name, name, "Const has the right name")
        harness.check(str(const.type), type, "Const has the right type")
        harness.ok(const.type.isPrimitive(), "All consts should be primitive")
        harness.check(str(const.value.type), str(const.type),
                      "Const's value has the same type as the type")
        harness.check(const.value.value, value, "Const value has the right value.")

    checkConst(consts[0], "::TestConsts::zero", "zero", "Byte", 0)
    checkConst(consts[1], "::TestConsts::b", "b", "Byte", -1)
    checkConst(consts[2], "::TestConsts::o", "o", "Octet", 2)
    checkConst(consts[3], "::TestConsts::s", "s", "Short", -3)
    checkConst(consts[4], "::TestConsts::us", "us", "UnsignedShort", 4)
    checkConst(consts[5], "::TestConsts::l", "l", "Long", -5)
    checkConst(consts[6], "::TestConsts::ul", "ul", "UnsignedLong", 6)
    checkConst(consts[7], "::TestConsts::ull", "ull", "UnsignedLongLong", 7)
    checkConst(consts[8], "::TestConsts::ll", "ll", "LongLong", -8)
    checkConst(consts[9], "::TestConsts::t", "t", "Boolean", True)
    checkConst(consts[10], "::TestConsts::f", "f", "Boolean", False)
    checkConst(consts[11], "::TestConsts::n", "n", "BooleanOrNull", None)
    checkConst(consts[12], "::TestConsts::nt", "nt", "BooleanOrNull", True)
    checkConst(consts[13], "::TestConsts::nf", "nf", "BooleanOrNull", False)

