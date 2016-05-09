# -*- coding: UTF-8 -*-

import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestByteString {
          attribute ByteString bs;
          attribute DOMString ds;
          };
    """)

    results = parser.finish();

    harness.ok(True, "TestByteString interface parsed without error.")
    
    harness.check(len(results), 1, "Should be one production")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    iface = results[0]
    harness.check(iface.identifier.QName(), "::TestByteString", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestByteString", "Interface has the right name")
    harness.check(iface.parent, None, "Interface has no parent")

    members = iface.members
    harness.check(len(members), 2, "Should be two productions")

    attr = members[0]
    harness.ok(isinstance(attr, WebIDL.IDLAttribute), "Should be an IDLAttribute")
    harness.check(attr.identifier.QName(), "::TestByteString::bs", "Attr has correct QName")
    harness.check(attr.identifier.name, "bs", "Attr has correct name")
    harness.check(str(attr.type), "ByteString", "Attr type is the correct name")
    harness.ok(attr.type.isByteString(), "Should be ByteString type")
    harness.ok(attr.type.isString(), "Should be String collective type")
    harness.ok(not attr.type.isDOMString(), "Should be not be DOMString type")

    # now check we haven't broken DOMStrings in the process.
    attr = members[1]
    harness.ok(isinstance(attr, WebIDL.IDLAttribute), "Should be an IDLAttribute")
    harness.check(attr.identifier.QName(), "::TestByteString::ds", "Attr has correct QName")
    harness.check(attr.identifier.name, "ds", "Attr has correct name")
    harness.check(str(attr.type), "String", "Attr type is the correct name")
    harness.ok(attr.type.isDOMString(), "Should be DOMString type")
    harness.ok(attr.type.isString(), "Should be String collective type")
    harness.ok(not attr.type.isByteString(), "Should be not be ByteString type")

    # Cannot represent constant ByteString in IDL.
    threw = False
    try:
        parser.parse("""
            interface ConstByteString {
              const ByteString foo = "hello"
              };
        """)
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should have thrown a WebIDL error")

    # Cannot have optional ByteStrings with default values
    threw = False
    try:
        parser.parse("""
            interface OptionalByteString {
              void passByteString(optional ByteString arg = "hello");
              };
        """)
        results2 = parser.finish();
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown a WebIDL error")

