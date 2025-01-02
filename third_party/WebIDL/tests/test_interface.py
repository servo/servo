import WebIDL


def WebIDLTest(parser, harness):
    parser.parse("interface Foo { };")
    results = parser.finish()
    harness.ok(True, "Empty interface parsed without error.")
    harness.check(len(results), 1, "Should be one production")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface), "Should be an IDLInterface")
    iface = results[0]
    harness.check(iface.identifier.QName(), "::Foo", "Interface has the right QName")
    harness.check(iface.identifier.name, "Foo", "Interface has the right name")
    harness.check(iface.parent, None, "Interface has no parent")

    parser.parse("interface Bar : Foo { };")
    results = parser.finish()
    harness.ok(True, "Empty interface parsed without error.")
    harness.check(len(results), 2, "Should be two productions")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface), "Should be an IDLInterface")
    iface = results[1]
    harness.check(iface.identifier.QName(), "::Bar", "Interface has the right QName")
    harness.check(iface.identifier.name, "Bar", "Interface has the right name")
    harness.ok(isinstance(iface.parent, WebIDL.IDLInterface), "Interface has a parent")

    parser = parser.reset()
    parser.parse(
        """
        interface QNameBase {
          attribute long foo;
        };

        interface QNameDerived : QNameBase {
          attribute long long foo;
          attribute byte bar;
        };
    """
    )
    results = parser.finish()
    harness.check(len(results), 2, "Should be two productions")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface), "Should be an IDLInterface")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface), "Should be an IDLInterface")
    harness.check(results[1].parent, results[0], "Inheritance chain is right")
    harness.check(len(results[0].members), 1, "Expect 1 productions")
    harness.check(len(results[1].members), 2, "Expect 2 productions")
    base = results[0]
    derived = results[1]
    harness.check(
        base.members[0].identifier.QName(),
        "::QNameBase::foo",
        "Member has the right QName",
    )
    harness.check(
        derived.members[0].identifier.QName(),
        "::QNameDerived::foo",
        "Member has the right QName",
    )
    harness.check(
        derived.members[1].identifier.QName(),
        "::QNameDerived::bar",
        "Member has the right QName",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A : B {};
            interface B : A {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow cycles in interface inheritance chains")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A : C {};
            interface C : B {};
            interface B : A {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should not allow indirect cycles in interface inheritance chains"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A;
            interface B : A {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should not allow inheriting from an interface that is only forward declared",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface A {
            constructor();
            constructor(long arg);
            readonly attribute boolean x;
            undefined foo();
        };
        partial interface A {
            readonly attribute boolean y;
            undefined foo(long arg);
        };
    """
    )
    results = parser.finish()
    harness.check(len(results), 2, "Should have two results with partial interface")
    iface = results[0]
    harness.check(
        len(iface.members), 3, "Should have three members with partial interface"
    )
    harness.check(
        iface.members[0].identifier.name,
        "x",
        "First member should be x with partial interface",
    )
    harness.check(
        iface.members[1].identifier.name,
        "foo",
        "Second member should be foo with partial interface",
    )
    harness.check(
        len(iface.members[1].signatures()),
        2,
        "Should have two foo signatures with partial interface",
    )
    harness.check(
        iface.members[2].identifier.name,
        "y",
        "Third member should be y with partial interface",
    )
    harness.check(
        len(iface.ctor().signatures()),
        2,
        "Should have two constructors with partial interface",
    )

    parser = parser.reset()
    parser.parse(
        """
        partial interface A {
            readonly attribute boolean y;
            undefined foo(long arg);
        };
        interface A {
            constructor();
            constructor(long arg);
            readonly attribute boolean x;
            undefined foo();
        };
    """
    )
    results = parser.finish()
    harness.check(
        len(results), 2, "Should have two results with reversed partial interface"
    )
    iface = results[1]
    harness.check(
        len(iface.members),
        3,
        "Should have three members with reversed partial interface",
    )
    harness.check(
        iface.members[0].identifier.name,
        "x",
        "First member should be x with reversed partial interface",
    )
    harness.check(
        iface.members[1].identifier.name,
        "foo",
        "Second member should be foo with reversed partial interface",
    )
    harness.check(
        len(iface.members[1].signatures()),
        2,
        "Should have two foo signatures with reversed partial interface",
    )
    harness.check(
        iface.members[2].identifier.name,
        "y",
        "Third member should be y with reversed partial interface",
    )
    harness.check(
        len(iface.ctor().signatures()),
        2,
        "Should have two constructors with reversed partial interface",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
                readonly attribute boolean x;
            };
            interface A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow two non-partial interfaces with the same name")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            partial interface A {
                readonly attribute boolean x;
            };
            partial interface A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Must have a non-partial interface for a given name")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary  A {
                boolean x;
            };
            partial interface A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow a name collision between partial interface "
        "and other object",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
                boolean x;
            };
            interface A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "Should not allow a name collision between interface " "and other object"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
                boolean x;
            };
            interface A;
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow a name collision between external interface "
        "and other object",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
                readonly attribute boolean x;
            };
            interface A;
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow a name collision between external interface " "and interface",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface A;
        interface A;
    """
    )
    results = parser.finish()
    harness.ok(
        len(results) == 1 and isinstance(results[0], WebIDL.IDLExternalInterface),
        "Should allow name collisions between external interface " "declarations",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [SomeRandomAnnotation]
            interface A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow unknown extended attributes on interfaces")

    parser = parser.reset()
    parser.parse(
        """
        [Global=Window, Exposed=Window] interface Window {};
        [Exposed=Window, LegacyWindowAlias=A]
        interface B {};
        [Exposed=Window, LegacyWindowAlias=(C, D)]
        interface E {};
    """
    )
    results = parser.finish()
    harness.check(
        results[1].legacyWindowAliases, ["A"], "Should support a single identifier"
    )
    harness.check(
        results[2].legacyWindowAliases, ["C", "D"], "Should support an identifier list"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [LegacyWindowAlias]
            interface A {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow [LegacyWindowAlias] with no value")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Exposed=Worker, LegacyWindowAlias=B]
            interface A {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow [LegacyWindowAlias] without Window exposure")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global=Window, Exposed=Window] interface Window {};
            [Exposed=Window]
            interface A {};
            [Exposed=Window, LegacyWindowAlias=A]
            interface B {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "Should not allow [LegacyWindowAlias] to conflict with other identifiers"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global=Window, Exposed=Window] interface Window {};
            [Exposed=Window, LegacyWindowAlias=A]
            interface B {};
            [Exposed=Window]
            interface A {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "Should not allow [LegacyWindowAlias] to conflict with other identifiers"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global=Window, Exposed=Window] interface Window {};
            [Exposed=Window, LegacyWindowAlias=A]
            interface B {};
            [Exposed=Window, LegacyWindowAlias=A]
            interface C {};
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "Should not allow [LegacyWindowAlias] to conflict with other identifiers"
    )
