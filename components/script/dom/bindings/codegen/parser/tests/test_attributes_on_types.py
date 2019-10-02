# Import the WebIDL module, so we can do isinstance checks and whatnot
import WebIDL

def WebIDLTest(parser, harness):
    # Basic functionality
    threw = False
    try:
        parser.parse("""
            typedef [EnforceRange] long Foo;
            typedef [Clamp] long Bar;
            typedef [TreatNullAs=EmptyString] DOMString Baz;
            dictionary A {
                required [EnforceRange] long a;
                required [Clamp] long b;
                [ChromeOnly, EnforceRange] long c;
                Foo d;
            };
            interface B {
                attribute Foo typedefFoo;
                attribute [EnforceRange] long foo;
                attribute [Clamp] long bar;
                attribute [TreatNullAs=EmptyString] DOMString baz;
                void method([EnforceRange] long foo, [Clamp] long bar,
                            [TreatNullAs=EmptyString] DOMString baz);
                void method2(optional [EnforceRange] long foo, optional [Clamp] long bar,
                             optional [TreatNullAs=EmptyString] DOMString baz);
            };
            interface Setlike {
                setlike<[Clamp] long>;
            };
            interface Maplike {
                maplike<[Clamp] long, [EnforceRange] long>;
            };
            interface Iterable {
                iterable<[Clamp] long, [EnforceRange] long>;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(not threw, "Should not have thrown on parsing normal")
    if not threw:
        harness.check(results[0].innerType.enforceRange, True, "Foo is [EnforceRange]")
        harness.check(results[1].innerType.clamp, True, "Bar is [Clamp]")
        harness.check(results[2].innerType.treatNullAsEmpty, True, "Baz is [TreatNullAs=EmptyString]")
        A = results[3]
        harness.check(A.members[0].type.enforceRange, True, "A.a is [EnforceRange]")
        harness.check(A.members[1].type.clamp, True, "A.b is [Clamp]")
        harness.check(A.members[2].type.enforceRange, True, "A.c is [EnforceRange]")
        harness.check(A.members[3].type.enforceRange, True, "A.d is [EnforceRange]")
        B = results[4]
        harness.check(B.members[0].type.enforceRange, True, "B.typedefFoo is [EnforceRange]")
        harness.check(B.members[1].type.enforceRange, True, "B.foo is [EnforceRange]")
        harness.check(B.members[2].type.clamp, True, "B.bar is [Clamp]")
        harness.check(B.members[3].type.treatNullAsEmpty, True, "B.baz is [TreatNullAs=EmptyString]")
        method = B.members[4].signatures()[0][1]
        harness.check(method[0].type.enforceRange, True, "foo argument of method is [EnforceRange]")
        harness.check(method[1].type.clamp, True, "bar argument of method is [Clamp]")
        harness.check(method[2].type.treatNullAsEmpty, True, "baz argument of method is [TreatNullAs=EmptyString]")
        method2 = B.members[5].signatures()[0][1]
        harness.check(method[0].type.enforceRange, True, "foo argument of method2 is [EnforceRange]")
        harness.check(method[1].type.clamp, True, "bar argument of method2 is [Clamp]")
        harness.check(method[2].type.treatNullAsEmpty, True, "baz argument of method2 is [TreatNullAs=EmptyString]")

    ATTRIBUTES = [("[Clamp]", "long"), ("[EnforceRange]", "long"), ("[TreatNullAs=EmptyString]", "DOMString")]
    TEMPLATES = [
        ("required dictionary members", """
            dictionary Foo {
                %s required %s foo;
            };
        """),
        ("optional arguments", """
            interface Foo {
                void foo(%s optional %s foo);
            };
        """),
        ("typedefs", """
            %s typedef %s foo;
        """),
        ("attributes", """
            interface Foo {
            %s attribute %s foo;
            };
        """),
        ("readonly attributes", """
            interface Foo {
                readonly attribute %s %s foo;
            };
        """),
        ("readonly unresolved attributes", """
            interface Foo {
              readonly attribute Bar baz;
            };
            typedef %s %s Bar;
        """)              
    ];

    for (name, template) in TEMPLATES:
        parser = parser.reset()
        threw = False
        try:
            parser.parse(template % ("", "long"))
            parser.finish()
        except:
            threw = True
        harness.ok(not threw, "Template for %s parses without attributes" % name)
        for (attribute, type) in ATTRIBUTES:
            parser = parser.reset()
            threw = False
            try:
                parser.parse(template % (attribute, type))
                parser.finish()
            except:
                threw = True
            harness.ok(threw,
                       "Should not allow %s on %s" % (attribute, name))

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [Clamp, EnforceRange] long Foo;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange]")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [EnforceRange, Clamp] long Foo;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange]")


    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [Clamp] long Foo;
            typedef [EnforceRange] Foo bar;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange] via typedefs")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [EnforceRange] long Foo;
            typedef [Clamp] Foo bar;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange] via typedefs")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [Clamp] DOMString Foo;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow [Clamp] on DOMString")


    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [EnforceRange] DOMString Foo;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow [EnforceRange] on DOMString")


    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [TreatNullAs=EmptyString] long Foo;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow [TreatNullAs] on long")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            typedef [TreatNullAs=EmptyString] JSString Foo;
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow [TreatNullAs] on JSString")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Foo {
               void foo([Clamp] Bar arg);
            };
            typedef long Bar;
        """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(not threw, "Should allow type attributes on unresolved types")
    harness.check(results[0].members[0].signatures()[0][1][0].type.clamp, True,
                  "Unresolved types with type attributes should correctly resolve with attributes")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Foo {
               void foo(Bar arg);
            };
            typedef [Clamp] long Bar;
        """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(not threw, "Should allow type attributes on typedefs")
    harness.check(results[0].members[0].signatures()[0][1][0].type.clamp, True,
                  "Unresolved types that resolve to typedefs with attributes should correctly resolve with attributes")
