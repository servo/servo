import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestBuiltins {
          attribute boolean b;
          attribute byte s8;
          attribute octet u8;
          attribute short s16;
          attribute unsigned short u16;
          attribute long s32;
          attribute unsigned long u32;
          attribute long long s64;
          attribute unsigned long long u64;
          attribute DOMTimeStamp ts;
        };
    """)

    results = parser.finish()

    harness.ok(True, "TestBuiltins interface parsed without error.")
    harness.check(len(results), 1, "Should be one production")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    iface = results[0]
    harness.check(iface.identifier.QName(), "::TestBuiltins", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestBuiltins", "Interface has the right name")
    harness.check(iface.parent, None, "Interface has no parent")

    members = iface.members
    harness.check(len(members), 10, "Should be one production")

    names = ["b", "s8", "u8", "s16", "u16", "s32", "u32", "s64", "u64", "ts"]
    types = ["Boolean", "Byte", "Octet", "Short", "UnsignedShort", "Long", "UnsignedLong", "LongLong", "UnsignedLongLong", "UnsignedLongLong"]
    for i in range(10):
        attr = members[i]
        harness.ok(isinstance(attr, WebIDL.IDLAttribute), "Should be an IDLAttribute")
        harness.check(attr.identifier.QName(), "::TestBuiltins::" + names[i], "Attr has correct QName")
        harness.check(attr.identifier.name, names[i], "Attr has correct name")
        harness.check(str(attr.type), types[i], "Attr type is the correct name")
        harness.ok(attr.type.isPrimitive(), "Should be a primitive type")
