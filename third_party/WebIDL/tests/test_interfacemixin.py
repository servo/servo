import WebIDL


def WebIDLTest(parser, harness):
    parser.parse("interface mixin Foo { };")
    results = parser.finish()
    harness.ok(True, "Empty interface mixin parsed without error.")
    harness.check(len(results), 1, "Should be one production")
    harness.ok(
        isinstance(results[0], WebIDL.IDLInterfaceMixin),
        "Should be an IDLInterfaceMixin",
    )
    mixin = results[0]
    harness.check(
        mixin.identifier.QName(), "::Foo", "Interface mixin has the right QName"
    )
    harness.check(mixin.identifier.name, "Foo", "Interface mixin has the right name")

    parser = parser.reset()
    parser.parse(
        """
        interface mixin QNameBase {
            const long foo = 3;
        };
    """
    )
    results = parser.finish()
    harness.check(len(results), 1, "Should be one productions")
    harness.ok(
        isinstance(results[0], WebIDL.IDLInterfaceMixin),
        "Should be an IDLInterfaceMixin",
    )
    harness.check(len(results[0].members), 1, "Expect 1 productions")
    mixin = results[0]
    harness.check(
        mixin.members[0].identifier.QName(),
        "::QNameBase::foo",
        "Member has the right QName",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface mixin A {
            readonly attribute boolean x;
            undefined foo();
        };
        partial interface mixin A {
            readonly attribute boolean y;
            undefined foo(long arg);
        };
    """
    )
    results = parser.finish()
    harness.check(
        len(results), 2, "Should have two results with partial interface mixin"
    )
    mixin = results[0]
    harness.check(
        len(mixin.members), 3, "Should have three members with partial interface mixin"
    )
    harness.check(
        mixin.members[0].identifier.name,
        "x",
        "First member should be x with partial interface mixin",
    )
    harness.check(
        mixin.members[1].identifier.name,
        "foo",
        "Second member should be foo with partial interface mixin",
    )
    harness.check(
        len(mixin.members[1].signatures()),
        2,
        "Should have two foo signatures with partial interface mixin",
    )
    harness.check(
        mixin.members[2].identifier.name,
        "y",
        "Third member should be y with partial interface mixin",
    )

    parser = parser.reset()
    parser.parse(
        """
        partial interface mixin A {
            readonly attribute boolean y;
            undefined foo(long arg);
        };
        interface mixin A {
            readonly attribute boolean x;
            undefined foo();
        };
    """
    )
    results = parser.finish()
    harness.check(
        len(results), 2, "Should have two results with reversed partial interface mixin"
    )
    mixin = results[1]
    harness.check(
        len(mixin.members),
        3,
        "Should have three members with reversed partial interface mixin",
    )
    harness.check(
        mixin.members[0].identifier.name,
        "x",
        "First member should be x with reversed partial interface mixin",
    )
    harness.check(
        mixin.members[1].identifier.name,
        "foo",
        "Second member should be foo with reversed partial interface mixin",
    )
    harness.check(
        len(mixin.members[1].signatures()),
        2,
        "Should have two foo signatures with reversed partial interface mixin",
    )
    harness.check(
        mixin.members[2].identifier.name,
        "y",
        "Third member should be y with reversed partial interface mixin",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface Interface {};
        interface mixin Mixin {
            attribute short x;
        };
        Interface includes Mixin;
    """
    )
    results = parser.finish()
    iface = results[0]
    harness.check(len(iface.members), 1, "Should merge members from mixins")
    harness.check(
        iface.members[0].identifier.name, "x", "Should merge members from mixins"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                readonly attribute boolean x;
            };
            interface mixin A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw, "Should not allow two non-partial interface mixins with the same name"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            partial interface mixin A {
                readonly attribute boolean x;
            };
            partial interface mixin A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Must have a non-partial interface mixin for a given name")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
                boolean x;
            };
            partial interface mixin A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw,
        "Should not allow a name collision between partial interface "
        "mixin and other object",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
                boolean x;
            };
            interface mixin A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw,
        "Should not allow a name collision between interface mixin " "and other object",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                readonly attribute boolean x;
            };
            interface A;
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw,
        "Should not allow a name collision between external interface "
        "and interface mixin",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [SomeRandomAnnotation]
            interface mixin A {
                readonly attribute boolean y;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw, "Should not allow unknown extended attributes on interface mixins"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                getter double (DOMString propertyName);
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should not allow getters on interface mixins")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                setter undefined (DOMString propertyName, double propertyValue);
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should not allow setters on interface mixins")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                deleter undefined (DOMString propertyName);
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should not allow deleters on interface mixins")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                legacycaller double compute(double x);
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should not allow legacycallers on interface mixins")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin A {
                inherit attribute x;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should not allow inherited attribute on interface mixins")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Interface {};
            interface NotMixin {
                attribute short x;
            };
            Interface includes NotMixin;
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should fail if the right side does not point an interface mixin")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin NotInterface {};
            interface mixin Mixin {
                attribute short x;
            };
            NotInterface includes Mixin;
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should fail if the left side does not point an interface")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin Mixin {
                iterable<DOMString>;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should fail if an interface mixin includes iterable")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin Mixin {
                setlike<DOMString>;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should fail if an interface mixin includes setlike")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface mixin Mixin {
                maplike<DOMString, DOMString>;
            };
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "Should fail if an interface mixin includes maplike")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Interface {
                attribute short attr;
            };
            interface mixin Mixin {
                attribute short attr;
            };
            Interface includes Mixin;
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw, "Should fail if the included mixin interface has duplicated member"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Interface {};
            interface mixin Mixin1 {
                attribute short attr;
            };
            interface mixin Mixin2 {
                attribute short attr;
            };
            Interface includes Mixin1;
            Interface includes Mixin2;
        """
        )
        results = parser.finish()
    except:
        threw = True
    harness.ok(
        threw, "Should fail if the included mixin interfaces have duplicated member"
    )

    parser = parser.reset()
    parser.parse(
        """
        [Global, Exposed=Window] interface Window {};
        [Global, Exposed=Worker] interface Worker {};
        [Exposed=Window]
        interface Base {};
        interface mixin Mixin {
            Base returnSelf();
        };
        Base includes Mixin;
    """
    )
    results = parser.finish()
    base = results[2]
    attr = base.members[0]
    harness.check(
        attr.exposureSet,
        set(["Window"]),
        "Should expose on globals where the base interfaces are exposed",
    )

    parser = parser.reset()
    parser.parse(
        """
        [Global, Exposed=Window] interface Window {};
        [Global, Exposed=Worker] interface Worker {};
        [Exposed=Window]
        interface Base {};
        [Exposed=Window]
        interface mixin Mixin {
            attribute short a;
        };
        Base includes Mixin;
    """
    )
    results = parser.finish()
    base = results[2]
    attr = base.members[0]
    harness.check(
        attr.exposureSet, set(["Window"]), "Should follow [Exposed] on interface mixin"
    )

    parser = parser.reset()
    parser.parse(
        """
        [Global, Exposed=Window] interface Window {};
        [Global, Exposed=Worker] interface Worker {};
        [Exposed=Window]
        interface Base1 {};
        [Exposed=Worker]
        interface Base2 {};
        interface mixin Mixin {
            attribute short a;
        };
        Base1 includes Mixin;
        Base2 includes Mixin;
    """
    )
    results = parser.finish()
    base = results[2]
    attr = base.members[0]
    harness.check(
        attr.exposureSet,
        set(["Window", "Worker"]),
        "Should expose on all globals where including interfaces are " "exposed",
    )
    base = results[3]
    attr = base.members[0]
    harness.check(
        attr.exposureSet,
        set(["Window", "Worker"]),
        "Should expose on all globals where including interfaces are " "exposed",
    )
