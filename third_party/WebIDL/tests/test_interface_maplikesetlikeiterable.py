import WebIDL


def WebIDLTest(parser, harness):
    def shouldPass(prefix, iface, expectedMembers, numProductions=1):
        p = parser.reset()
        p.parse(iface)
        results = p.finish()
        harness.check(
            len(results),
            numProductions,
            "%s - Should have production count %d" % (prefix, numProductions),
        )
        harness.ok(
            isinstance(results[0], WebIDL.IDLInterface),
            "%s - Should be an IDLInterface" % (prefix),
        )
        # Make a copy, since we plan to modify it
        expectedMembers = list(expectedMembers)
        for m in results[0].members:
            name = m.identifier.name
            if m.isMethod() and m.isStatic():
                # None of the expected members are static methods, so ignore those.
                harness.ok(True, "%s - %s - Should be a %s" % (prefix, name, type(m)))
            elif (name, type(m)) in expectedMembers:
                harness.ok(True, "%s - %s - Should be a %s" % (prefix, name, type(m)))
                expectedMembers.remove((name, type(m)))
            else:
                harness.ok(
                    False,
                    "%s - %s - Unknown symbol of type %s" % (prefix, name, type(m)),
                )
        # A bit of a hoop because we can't generate the error string if we pass
        if len(expectedMembers) == 0:
            harness.ok(True, "Found all the members")
        else:
            harness.ok(
                False,
                "Expected member not found: %s of type %s"
                % (expectedMembers[0][0], expectedMembers[0][1]),
            )
        return results

    def shouldFail(prefix, iface):
        try:
            p = parser.reset()
            p.parse(iface)
            p.finish()
            harness.ok(False, prefix + " - Interface passed when should've failed")
        except WebIDL.WebIDLError:
            harness.ok(True, prefix + " - Interface failed as expected")
        except Exception as e:
            harness.ok(
                False,
                prefix
                + " - Interface failed but not as a WebIDLError exception: %s" % e,
            )

    iterableMembers = [
        (x, WebIDL.IDLMethod) for x in ["entries", "keys", "values", "forEach"]
    ]
    setROMembers = (
        [(x, WebIDL.IDLMethod) for x in ["has"]]
        + [("__setlike", WebIDL.IDLMaplikeOrSetlike)]
        + iterableMembers
    )
    setROMembers.extend([("size", WebIDL.IDLAttribute)])
    setRWMembers = [
        (x, WebIDL.IDLMethod) for x in ["add", "clear", "delete"]
    ] + setROMembers
    mapROMembers = (
        [(x, WebIDL.IDLMethod) for x in ["get", "has"]]
        + [("__maplike", WebIDL.IDLMaplikeOrSetlike)]
        + iterableMembers
    )
    mapROMembers.extend([("size", WebIDL.IDLAttribute)])
    mapRWMembers = [
        (x, WebIDL.IDLMethod) for x in ["set", "clear", "delete"]
    ] + mapROMembers

    # OK, now that we've used iterableMembers to set up the above, append
    # __iterable to it for the iterable<> case.
    iterableMembers.append(("__iterable", WebIDL.IDLIterable))

    asyncIterableMembers = [
        (x, WebIDL.IDLMethod) for x in ["entries", "keys", "values"]
    ]
    asyncIterableMembers.append(("__iterable", WebIDL.IDLAsyncIterable))

    valueIterableMembers = [("__iterable", WebIDL.IDLIterable)]
    valueIterableMembers.append(("__indexedgetter", WebIDL.IDLMethod))
    valueIterableMembers.append(("length", WebIDL.IDLAttribute))

    valueAsyncIterableMembers = [("__iterable", WebIDL.IDLAsyncIterable)]
    valueAsyncIterableMembers.append(("values", WebIDL.IDLMethod))

    disallowedIterableNames = [
        ("keys", WebIDL.IDLMethod),
        ("entries", WebIDL.IDLMethod),
        ("values", WebIDL.IDLMethod),
    ]
    disallowedMemberNames = [
        ("forEach", WebIDL.IDLMethod),
        ("has", WebIDL.IDLMethod),
        ("size", WebIDL.IDLAttribute),
    ] + disallowedIterableNames
    mapDisallowedMemberNames = [("get", WebIDL.IDLMethod)] + disallowedMemberNames
    disallowedNonMethodNames = [
        ("clear", WebIDL.IDLMethod),
        ("delete", WebIDL.IDLMethod),
    ]
    mapDisallowedNonMethodNames = [("set", WebIDL.IDLMethod)] + disallowedNonMethodNames
    setDisallowedNonMethodNames = [("add", WebIDL.IDLMethod)] + disallowedNonMethodNames
    unrelatedMembers = [
        ("unrelatedAttribute", WebIDL.IDLAttribute),
        ("unrelatedMethod", WebIDL.IDLMethod),
    ]

    #
    # Simple Usage Tests
    #

    shouldPass(
        "Iterable (key only)",
        """
               interface Foo1 {
               iterable<long>;
               readonly attribute unsigned long length;
               getter long(unsigned long index);
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        valueIterableMembers + unrelatedMembers,
    )

    shouldPass(
        "Iterable (key only) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               iterable<long>;
               readonly attribute unsigned long length;
               getter long(unsigned long index);
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        valueIterableMembers,
        numProductions=2,
    )

    shouldPass(
        "Iterable (key and value)",
        """
               interface Foo1 {
               iterable<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        iterableMembers + unrelatedMembers,
        # numProductions == 2 because of the generated iterator iface,
        numProductions=2,
    )

    shouldPass(
        "Iterable (key and value) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               iterable<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        iterableMembers,
        # numProductions == 3 because of the generated iterator iface,
        numProductions=3,
    )

    shouldPass(
        "Async iterable (key only)",
        """
               interface Foo1 {
               async iterable<long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        valueAsyncIterableMembers + unrelatedMembers,
        # numProductions == 2 because of the generated iterator iface,
        numProductions=2,
    )

    shouldPass(
        "Async iterable (key only) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               async iterable<long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        valueAsyncIterableMembers,
        # numProductions == 3 because of the generated iterator iface,
        numProductions=3,
    )

    shouldPass(
        "Async iterable with argument (key only)",
        """
               interface Foo1 {
               async iterable<long>(optional long foo);
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        valueAsyncIterableMembers + unrelatedMembers,
        # numProductions == 2 because of the generated iterator iface,
        numProductions=2,
    )

    shouldPass(
        "Async iterable (key and value)",
        """
               interface Foo1 {
               async iterable<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        asyncIterableMembers + unrelatedMembers,
        # numProductions == 2 because of the generated iterator iface,
        numProductions=2,
    )

    shouldPass(
        "Async iterable (key and value) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               async iterable<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        asyncIterableMembers,
        # numProductions == 3 because of the generated iterator iface,
        numProductions=3,
    )

    shouldPass(
        "Async iterable with argument (key and value)",
        """
               interface Foo1 {
               async iterable<long, long>(optional long foo);
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        asyncIterableMembers + unrelatedMembers,
        # numProductions == 2 because of the generated iterator iface,
        numProductions=2,
    )

    shouldPass(
        "Maplike (readwrite)",
        """
               interface Foo1 {
               maplike<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        mapRWMembers + unrelatedMembers,
    )

    shouldPass(
        "Maplike (readwrite) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               maplike<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        mapRWMembers,
        numProductions=2,
    )

    shouldPass(
        "Maplike (readwrite)",
        """
               interface Foo1 {
               maplike<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        mapRWMembers + unrelatedMembers,
    )

    shouldPass(
        "Maplike (readwrite) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               maplike<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        mapRWMembers,
        numProductions=2,
    )

    shouldPass(
        "Maplike (readonly)",
        """
               interface Foo1 {
               readonly maplike<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        mapROMembers + unrelatedMembers,
    )

    shouldPass(
        "Maplike (readonly) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               readonly maplike<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        mapROMembers,
        numProductions=2,
    )

    shouldPass(
        "Setlike (readwrite)",
        """
               interface Foo1 {
               setlike<long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        setRWMembers + unrelatedMembers,
    )

    shouldPass(
        "Setlike (readwrite) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               setlike<long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        setRWMembers,
        numProductions=2,
    )

    shouldPass(
        "Setlike (readonly)",
        """
               interface Foo1 {
               readonly setlike<long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        setROMembers + unrelatedMembers,
    )

    shouldPass(
        "Setlike (readonly) inheriting from parent",
        """
               interface Foo1 : Foo2 {
               readonly setlike<long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """,
        setROMembers,
        numProductions=2,
    )

    shouldPass(
        "Inheritance of maplike/setlike",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               """,
        mapRWMembers,
        numProductions=2,
    )

    shouldFail(
        "JS Implemented maplike interface",
        """
               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1"]
               interface Foo1 {
               constructor();
               setlike<long>;
               };
               """,
    )

    shouldFail(
        "JS Implemented maplike interface",
        """
               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1"]
               interface Foo1 {
               constructor();
               maplike<long, long>;
               };
               """,
    )

    #
    # Multiple maplike/setlike tests
    #

    shouldFail(
        "Two maplike/setlikes on same interface",
        """
               interface Foo1 {
               setlike<long>;
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Two iterable/setlikes on same interface",
        """
               interface Foo1 {
               iterable<long>;
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Two iterables on same interface",
        """
               interface Foo1 {
               iterable<long>;
               iterable<long, long>;
               };
               """,
    )

    shouldFail(
        "Two iterables on same interface",
        """
               interface Foo1 {
               iterable<long>;
               async iterable<long>;
               };
               """,
    )

    shouldFail(
        "Two iterables on same interface",
        """
               interface Foo1 {
               async iterable<long>;
               async iterable<long, long>;
               };
               """,
    )

    shouldFail(
        "Async iterable with non-optional arguments",
        """
               interface Foo1 {
               async iterable<long>(long foo);
               };
               """,
    )

    shouldFail(
        "Async iterable with non-optional arguments",
        """
               interface Foo1 {
               async iterable<long>(optional long foo, long bar);
               };
               """,
    )

    shouldFail(
        "Async iterable with non-optional arguments",
        """
               interface Foo1 {
               async iterable<long, long>(long foo);
               };
               """,
    )

    shouldFail(
        "Two maplike/setlikes in partials",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               partial interface Foo1 {
               setlike<long>;
               };
               """,
    )

    shouldFail(
        "Conflicting maplike/setlikes across inheritance",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               setlike<long>;
               };
               """,
    )

    shouldFail(
        "Conflicting maplike/iterable across inheritance",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               iterable<long>;
               };
               """,
    )

    shouldFail(
        "Conflicting maplike/setlikes across multistep inheritance",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               setlike<long>;
               };
               """,
    )

    #
    # Member name collision tests
    #

    def testConflictingMembers(
        likeMember, conflict, expectedMembers, methodPasses, numProductions=1
    ):
        """
        Tests for maplike/setlike member generation against conflicting member
        names. If methodPasses is True, this means we expect the interface to
        pass in the case of method shadowing, and expectedMembers should be the
        list of interface members to check against on the passing interface.

        """
        (conflictName, conflictType) = conflict
        if methodPasses:
            shouldPass(
                "Conflicting method: %s and %s" % (likeMember, conflictName),
                """
                       interface Foo1 {
                       %s;
                       [Throws]
                       undefined %s(long test1, double test2, double test3);
                       };
                       """
                % (likeMember, conflictName),
                expectedMembers,
            )
        else:
            shouldFail(
                "Conflicting method: %s and %s" % (likeMember, conflictName),
                """
                       interface Foo1 {
                       %s;
                       [Throws]
                       undefined %s(long test1, double test2, double test3);
                       };
                       """
                % (likeMember, conflictName),
            )
        # Inherited conflicting methods should ALWAYS fail
        shouldFail(
            "Conflicting inherited method: %s and %s" % (likeMember, conflictName),
            """
                   interface Foo1 {
                   undefined %s(long test1, double test2, double test3);
                   };
                   interface Foo2 : Foo1 {
                   %s;
                   };
                   """
            % (conflictName, likeMember),
        )
        if conflictType == WebIDL.IDLAttribute:
            shouldFail(
                "Conflicting static method: %s and %s" % (likeMember, conflictName),
                """
                       interface Foo1 {
                       %s;
                       static undefined %s(long test1, double test2, double test3);
                       };
                       """
                % (likeMember, conflictName),
            )
        else:
            shouldPass(
                "Conflicting static method: %s and %s" % (likeMember, conflictName),
                """
                       interface Foo1 {
                       %s;
                       static undefined %s(long test1, double test2, double test3);
                       };
                       """
                % (likeMember, conflictName),
                expectedMembers,
                numProductions=numProductions,
            )
        shouldFail(
            "Conflicting attribute: %s and %s" % (likeMember, conflictName),
            """
                   interface Foo1 {
                   %s
                   attribute double %s;
                   };
                   """
            % (likeMember, conflictName),
        )
        shouldFail(
            "Conflicting const: %s and %s" % (likeMember, conflictName),
            """
                   interface Foo1 {
                   %s;
                   const double %s = 0;
                   };
                   """
            % (likeMember, conflictName),
        )
        shouldFail(
            "Conflicting static attribute: %s and %s" % (likeMember, conflictName),
            """
                   interface Foo1 {
                   %s;
                   static attribute long %s;
                   };
                   """
            % (likeMember, conflictName),
        )

    for member in disallowedIterableNames:
        testConflictingMembers(
            "iterable<long, long>", member, iterableMembers, False, numProductions=2
        )
    for member in mapDisallowedMemberNames:
        testConflictingMembers("maplike<long, long>", member, mapRWMembers, False)
    for member in disallowedMemberNames:
        testConflictingMembers("setlike<long>", member, setRWMembers, False)
    for member in mapDisallowedNonMethodNames:
        testConflictingMembers("maplike<long, long>", member, mapRWMembers, True)
    for member in setDisallowedNonMethodNames:
        testConflictingMembers("setlike<long>", member, setRWMembers, True)

    shouldPass(
        "Inheritance of maplike/setlike with child member collision",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               undefined entries();
               };
               """,
        mapRWMembers,
        numProductions=2,
    )

    shouldPass(
        "Inheritance of multi-level maplike/setlike with child member collision",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               undefined entries();
               };
               """,
        mapRWMembers,
        numProductions=3,
    )

    shouldFail(
        "Maplike interface with mixin member collision",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface mixin Foo2 {
               undefined entries();
               };
               Foo1 includes Foo2;
               """,
    )

    shouldPass(
        "Inherited Maplike interface with consequential interface member collision",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface mixin Foo2 {
               undefined entries();
               };
               interface Foo3 : Foo1 {
               };
               Foo3 includes Foo2;
               """,
        mapRWMembers,
        numProductions=4,
    )

    shouldFail(
        "Inheritance of name collision with child maplike/setlike",
        """
               interface Foo1 {
               undefined entries();
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Inheritance of multi-level name collision with child maplike/setlike",
        """
               interface Foo1 {
               undefined entries();
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               maplike<long, long>;
               };
               """,
    )

    shouldPass(
        "Inheritance of attribute collision with parent maplike/setlike",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               attribute double size;
               };
               """,
        mapRWMembers,
        numProductions=2,
    )

    shouldPass(
        "Inheritance of multi-level attribute collision with parent maplike/setlike",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               attribute double size;
               };
               """,
        mapRWMembers,
        numProductions=3,
    )

    shouldFail(
        "Inheritance of attribute collision with child maplike/setlike",
        """
               interface Foo1 {
               attribute double size;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Inheritance of multi-level attribute collision with child maplike/setlike",
        """
               interface Foo1 {
               attribute double size;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Inheritance of attribute/rw function collision with child maplike/setlike",
        """
               interface Foo1 {
               attribute double set;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Inheritance of const/rw function collision with child maplike/setlike",
        """
               interface Foo1 {
               const double set = 0;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """,
    )

    shouldPass(
        "Inheritance of rw function with same name in child maplike/setlike",
        """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               undefined clear();
               };
               """,
        mapRWMembers,
        numProductions=2,
    )

    shouldFail(
        "Inheritance of unforgeable attribute collision with child maplike/setlike",
        """
               interface Foo1 {
               [LegacyUnforgeable]
               attribute double size;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """,
    )

    shouldFail(
        "Inheritance of multi-level unforgeable attribute collision with child maplike/setlike",
        """
               interface Foo1 {
               [LegacyUnforgeable]
               attribute double size;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               maplike<long, long>;
               };
               """,
    )

    shouldPass(
        "Interface with readonly allowable overrides",
        """
               interface Foo1 {
               readonly setlike<long>;
               readonly attribute boolean clear;
               };
               """,
        setROMembers + [("clear", WebIDL.IDLAttribute)],
    )

    r = shouldPass(
        "Check proper override of clear/delete/set",
        """
                   interface Foo1 {
                   maplike<long, long>;
                   long clear(long a, long b, double c, double d);
                   long set(long a, long b, double c, double d);
                   long delete(long a, long b, double c, double d);
                   };
                   """,
        mapRWMembers,
    )

    for m in r[0].members:
        if m.identifier.name in ["clear", "set", "delete"]:
            harness.ok(m.isMethod(), "%s should be a method" % m.identifier.name)
            harness.check(
                m.maxArgCount, 4, "%s should have 4 arguments" % m.identifier.name
            )
            harness.ok(
                not m.isMaplikeOrSetlikeOrIterableMethod(),
                "%s should not be a maplike/setlike function" % m.identifier.name,
            )
