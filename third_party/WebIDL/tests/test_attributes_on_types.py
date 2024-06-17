import WebIDL


def WebIDLTest(parser, harness):
    # Basic functionality
    threw = False
    try:
        parser.parse(
            """
            typedef [EnforceRange] long Foo;
            typedef [Clamp] long Bar;
            typedef [LegacyNullToEmptyString] DOMString Baz;
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
                attribute [LegacyNullToEmptyString] DOMString baz;
                undefined method([EnforceRange] long foo, [Clamp] long bar,
                                 [LegacyNullToEmptyString] DOMString baz);
                undefined method2(optional [EnforceRange] long foo, optional [Clamp] long bar,
                                  optional [LegacyNullToEmptyString] DOMString baz);
                undefined method3(optional [LegacyNullToEmptyString] UTF8String foo = "");
            };
            interface C {
                attribute [EnforceRange] long? foo;
                attribute [Clamp] long? bar;
                undefined method([EnforceRange] long? foo, [Clamp] long? bar);
                undefined method2(optional [EnforceRange] long? foo, optional [Clamp] long? bar);
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
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(not threw, "Should not have thrown on parsing normal")
    if not threw:
        harness.check(
            results[0].innerType.hasEnforceRange(), True, "Foo is [EnforceRange]"
        )
        harness.check(results[1].innerType.hasClamp(), True, "Bar is [Clamp]")
        harness.check(
            results[2].innerType.legacyNullToEmptyString,
            True,
            "Baz is [LegacyNullToEmptyString]",
        )
        A = results[3]
        harness.check(
            A.members[0].type.hasEnforceRange(), True, "A.a is [EnforceRange]"
        )
        harness.check(A.members[1].type.hasClamp(), True, "A.b is [Clamp]")
        harness.check(
            A.members[2].type.hasEnforceRange(), True, "A.c is [EnforceRange]"
        )
        harness.check(
            A.members[3].type.hasEnforceRange(), True, "A.d is [EnforceRange]"
        )
        B = results[4]
        harness.check(
            B.members[0].type.hasEnforceRange(), True, "B.typedefFoo is [EnforceRange]"
        )
        harness.check(
            B.members[1].type.hasEnforceRange(), True, "B.foo is [EnforceRange]"
        )
        harness.check(B.members[2].type.hasClamp(), True, "B.bar is [Clamp]")
        harness.check(
            B.members[3].type.legacyNullToEmptyString,
            True,
            "B.baz is [LegacyNullToEmptyString]",
        )
        method = B.members[4].signatures()[0][1]
        harness.check(
            method[0].type.hasEnforceRange(),
            True,
            "foo argument of method is [EnforceRange]",
        )
        harness.check(
            method[1].type.hasClamp(), True, "bar argument of method is [Clamp]"
        )
        harness.check(
            method[2].type.legacyNullToEmptyString,
            True,
            "baz argument of method is [LegacyNullToEmptyString]",
        )
        method2 = B.members[5].signatures()[0][1]
        harness.check(
            method2[0].type.hasEnforceRange(),
            True,
            "foo argument of method2 is [EnforceRange]",
        )
        harness.check(
            method2[1].type.hasClamp(), True, "bar argument of method2 is [Clamp]"
        )
        harness.check(
            method2[2].type.legacyNullToEmptyString,
            True,
            "baz argument of method2 is [LegacyNullToEmptyString]",
        )

        method3 = B.members[6].signatures()[0][1]
        harness.check(
            method3[0].type.legacyNullToEmptyString,
            True,
            "bar argument of method2 is [LegacyNullToEmptyString]",
        )
        harness.check(
            method3[0].defaultValue.type.isUTF8String(),
            True,
            "default value of bar argument of method2 is correctly coerced to UTF8String",
        )

        C = results[5]
        harness.ok(C.members[0].type.nullable(), "C.foo is nullable")
        harness.ok(C.members[0].type.hasEnforceRange(), "C.foo has [EnforceRange]")
        harness.ok(C.members[1].type.nullable(), "C.bar is nullable")
        harness.ok(C.members[1].type.hasClamp(), "C.bar has [Clamp]")
        method = C.members[2].signatures()[0][1]
        harness.ok(method[0].type.nullable(), "foo argument of method is nullable")
        harness.ok(
            method[0].type.hasEnforceRange(),
            "foo argument of method has [EnforceRange]",
        )
        harness.ok(method[1].type.nullable(), "bar argument of method is nullable")
        harness.ok(method[1].type.hasClamp(), "bar argument of method has [Clamp]")
        method2 = C.members[3].signatures()[0][1]
        harness.ok(method2[0].type.nullable(), "foo argument of method2 is nullable")
        harness.ok(
            method2[0].type.hasEnforceRange(),
            "foo argument of method2 has [EnforceRange]",
        )
        harness.ok(method2[1].type.nullable(), "bar argument of method2 is nullable")
        harness.ok(method2[1].type.hasClamp(), "bar argument of method2 has [Clamp]")

    # Test [AllowShared]
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [AllowShared] ArrayBufferView Foo;
            dictionary A {
                required [AllowShared] ArrayBufferView a;
                [ChromeOnly, AllowShared] ArrayBufferView b;
                Foo c;
            };
            interface B {
                attribute Foo typedefFoo;
                attribute [AllowShared] ArrayBufferView foo;
                undefined method([AllowShared] ArrayBufferView foo);
                undefined method2(optional [AllowShared] ArrayBufferView foo);
            };
            interface C {
                attribute [AllowShared] ArrayBufferView? foo;
                undefined method([AllowShared] ArrayBufferView? foo);
                undefined method2(optional [AllowShared] ArrayBufferView? foo);
            };
            interface Setlike {
                setlike<[AllowShared] ArrayBufferView>;
            };
            interface Maplike {
                maplike<[Clamp] long, [AllowShared] ArrayBufferView>;
            };
            interface Iterable {
                iterable<[Clamp] long, [AllowShared] ArrayBufferView>;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(not threw, "Should not have thrown on parsing normal")
    if not threw:
        harness.ok(results[0].innerType.hasAllowShared(), "Foo is [AllowShared]")
        A = results[1]
        harness.ok(A.members[0].type.hasAllowShared(), "A.a is [AllowShared]")
        harness.ok(A.members[1].type.hasAllowShared(), "A.b is [AllowShared]")
        harness.ok(A.members[2].type.hasAllowShared(), "A.c is [AllowShared]")
        B = results[2]
        harness.ok(B.members[0].type.hasAllowShared(), "B.typedefFoo is [AllowShared]")
        harness.ok(B.members[1].type.hasAllowShared(), "B.foo is [AllowShared]")
        method = B.members[2].signatures()[0][1]
        harness.ok(
            method[0].type.hasAllowShared(), "foo argument of method is [AllowShared]"
        )
        method2 = B.members[3].signatures()[0][1]
        harness.ok(
            method2[0].type.hasAllowShared(), "foo argument of method2 is [AllowShared]"
        )
        C = results[3]
        harness.ok(C.members[0].type.nullable(), "C.foo is nullable")
        harness.ok(C.members[0].type.hasAllowShared(), "C.foo is [AllowShared]")
        method = C.members[1].signatures()[0][1]
        harness.ok(method[0].type.nullable(), "foo argument of method is nullable")
        harness.ok(
            method[0].type.hasAllowShared(), "foo argument of method is [AllowShared]"
        )
        method2 = C.members[2].signatures()[0][1]
        harness.ok(method2[0].type.nullable(), "foo argument of method2 is nullable")
        harness.ok(
            method2[0].type.hasAllowShared(), "foo argument of method2 is [AllowShared]"
        )

    ATTRIBUTES = [
        ("[Clamp]", "long"),
        ("[EnforceRange]", "long"),
        ("[LegacyNullToEmptyString]", "DOMString"),
        ("[AllowShared]", "ArrayBufferView"),
    ]
    TEMPLATES = [
        (
            "required dictionary members",
            """
            dictionary Foo {
                %s required %s foo;
            };
        """,
        ),
        (
            "optional arguments",
            """
            interface Foo {
                undefined foo(%s optional %s foo);
            };
        """,
        ),
        (
            "typedefs",
            """
            %s typedef %s foo;
        """,
        ),
        (
            "attributes",
            """
            interface Foo {
            %s attribute %s foo;
            };
        """,
        ),
        (
            "readonly attributes",
            """
            interface Foo {
                readonly attribute %s %s foo;
            };
        """,
        ),
        (
            "readonly unresolved attributes",
            """
            interface Foo {
              readonly attribute Bar baz;
            };
            typedef %s %s Bar;
        """,
        ),
        (
            "method",
            """
            interface Foo {
              %s %s foo();
            };
        """,
        ),
        (
            "interface",
            """
            %s
            interface Foo {
              attribute %s foo;
            };
        """,
        ),
        (
            "partial interface",
            """
            interface Foo {
              undefined foo();
            };
            %s
            partial interface Foo {
              attribute %s bar;
            };
        """,
        ),
        (
            "interface mixin",
            """
            %s
            interface mixin Foo {
              attribute %s foo;
            };
        """,
        ),
        (
            "namespace",
            """
            %s
            namespace Foo {
              attribute %s foo;
            };
        """,
        ),
        (
            "partial namespace",
            """
            namespace Foo {
              undefined foo();
            };
            %s
            partial namespace Foo {
              attribute %s bar;
            };
        """,
        ),
        (
            "dictionary",
            """
            %s
            dictionary Foo {
              %s foo;
            };
        """,
        ),
    ]

    for name, template in TEMPLATES:
        parser = parser.reset()
        threw = False
        try:
            parser.parse(template % ("", "long"))
            parser.finish()
        except WebIDL.WebIDLError:
            threw = True
        harness.ok(not threw, "Template for %s parses without attributes" % name)
        for attribute, type in ATTRIBUTES:
            parser = parser.reset()
            threw = False
            try:
                parser.parse(template % (attribute, type))
                parser.finish()
            except WebIDL.WebIDLError:
                threw = True
            harness.ok(threw, "Should not allow %s on %s" % (attribute, name))

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [Clamp, EnforceRange] long Foo;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange]")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [EnforceRange, Clamp] long Foo;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange]")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [Clamp] long Foo;
            typedef [EnforceRange] Foo bar;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange] via typedefs")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [EnforceRange] long Foo;
            typedef [Clamp] Foo bar;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow mixing [Clamp] and [EnforceRange] via typedefs")

    TYPES = [
        "DOMString",
        "unrestricted float",
        "float",
        "unrestricted double",
        "double",
    ]

    for type in TYPES:
        parser = parser.reset()
        threw = False
        try:
            parser.parse(
                """
                typedef [Clamp] %s Foo;
            """
                % type
            )
            parser.finish()
        except WebIDL.WebIDLError:
            threw = True

        harness.ok(threw, "Should not allow [Clamp] on %s" % type)

        parser = parser.reset()
        threw = False
        try:
            parser.parse(
                """
                typedef [EnforceRange] %s Foo;
            """
                % type
            )
            parser.finish()
        except WebIDL.WebIDLError:
            threw = True

        harness.ok(threw, "Should not allow [EnforceRange] on %s" % type)

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [LegacyNullToEmptyString] long Foo;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow [LegacyNullToEmptyString] on long")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [LegacyNullToEmptyString] JSString Foo;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow [LegacyNullToEmptyString] on JSString")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [LegacyNullToEmptyString] DOMString? Foo;
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should not allow [LegacyNullToEmptyString] on nullable DOMString"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [AllowShared] DOMString Foo;
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "[AllowShared] only allowed on buffer source types")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            typedef [AllowShared=something] ArrayBufferView Foo;
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "[AllowShared] must take no arguments")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
               undefined foo([Clamp] Bar arg);
            };
            typedef long Bar;
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(not threw, "Should allow type attributes on unresolved types")
    harness.check(
        results[0].members[0].signatures()[0][1][0].type.hasClamp(),
        True,
        "Unresolved types with type attributes should correctly resolve with attributes",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
               undefined foo(Bar arg);
            };
            typedef [Clamp] long Bar;
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(not threw, "Should allow type attributes on typedefs")
    harness.check(
        results[0].members[0].signatures()[0][1][0].type.hasClamp(),
        True,
        "Unresolved types that resolve to typedefs with attributes should correctly resolve with "
        "attributes",
    )
