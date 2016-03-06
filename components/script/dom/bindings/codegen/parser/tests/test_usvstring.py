# -*- coding: UTF-8 -*-

import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestUSVString {
          attribute USVString svs;
        };
    """)

    results = parser.finish();

    harness.check(len(results), 1, "Should be one production")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    iface = results[0]
    harness.check(iface.identifier.QName(), "::TestUSVString",
                  "Interface has the right QName")
    harness.check(iface.identifier.name, "TestUSVString",
                  "Interface has the right name")
    harness.check(iface.parent, None, "Interface has no parent")

    members = iface.members
    harness.check(len(members), 1, "Should be one member")

    attr = members[0]
    harness.ok(isinstance(attr, WebIDL.IDLAttribute), "Should be an IDLAttribute")
    harness.check(attr.identifier.QName(), "::TestUSVString::svs",
                  "Attr has correct QName")
    harness.check(attr.identifier.name, "svs", "Attr has correct name")
    harness.check(str(attr.type), "USVString",
                  "Attr type is the correct name")
    harness.ok(attr.type.isUSVString(), "Should be USVString type")
    harness.ok(attr.type.isString(), "Should be String collective type")
    harness.ok(not attr.type.isDOMString(), "Should be not be DOMString type")
