import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("interface Foo { };")
    results = parser.finish()
    harness.ok(True, "Empty interface parsed without error.")
    harness.check(len(results), 1, "Should be one production")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    iface = results[0]
    harness.check(iface.identifier.QName(), "::Foo", "Interface has the right QName")
    harness.check(iface.identifier.name, "Foo", "Interface has the right name")
    harness.check(iface.parent, None, "Interface has no parent")

    parser.parse("interface Bar : Foo { };")
    results = parser.finish()
    harness.ok(True, "Empty interface parsed without error.")
    harness.check(len(results), 2, "Should be two productions")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    iface = results[1]
    harness.check(iface.identifier.QName(), "::Bar", "Interface has the right QName")
    harness.check(iface.identifier.name, "Bar", "Interface has the right name")
    harness.ok(isinstance(iface.parent, WebIDL.IDLInterface),
               "Interface has a parent")

    parser = parser.reset()
    parser.parse("""
        interface QNameBase {
          attribute long foo;
        };

        interface QNameDerived : QNameBase {
          attribute long long foo;
          attribute byte bar;          
        };
    """)
    results = parser.finish()
    harness.check(len(results), 2, "Should be two productions")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.check(results[1].parent, results[0], "Inheritance chain is right")
    harness.check(len(results[0].members), 1, "Expect 1 productions")
    harness.check(len(results[1].members), 2, "Expect 2 productions")
    base = results[0]
    derived = results[1]
    harness.check(base.members[0].identifier.QName(), "::QNameBase::foo",
                  "Member has the right QName")
    harness.check(derived.members[0].identifier.QName(), "::QNameDerived::foo",
                  "Member has the right QName")
    harness.check(derived.members[1].identifier.QName(), "::QNameDerived::bar",
                  "Member has the right QName")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A : B {};
            interface B : A {};
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow cycles in interface inheritance chains")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A : C {};
            interface C : B {};
            interface B : A {};
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow indirect cycles in interface inheritance chains")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A {};
            interface B {};
            A implements B;
            B implements A;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow cycles via implements")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A {};
            interface C {};
            interface B {};
            A implements C;
            C implements B;
            B implements A;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow indirect cycles via implements")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A : B {};
            interface B {};
            B implements A;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow inheriting from an interface that implements us")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A : B {};
            interface B {};
            interface C {};
            B implements C;
            C implements A;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow inheriting from an interface that indirectly implements us")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A : B {};
            interface B : C {};
            interface C {};
            C implements A;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow indirectly inheriting from an interface that implements us")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A : B {};
            interface B : C {};
            interface C {};
            interface D {};
            C implements D;
            D implements A;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow indirectly inheriting from an interface that indirectly implements us")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface A;
            interface B : A {};
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow inheriting from an interface that is only forward declared")
