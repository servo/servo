import WebIDL
import traceback
def WebIDLTest(parser, harness):

    def shouldPass(prefix, iface, expectedMembers, numProductions=1):
        p = parser.reset()
        p.parse(iface)
        results = p.finish()
        harness.check(len(results), numProductions,
                      "%s - Should have production count %d" % (prefix, numProductions))
        harness.ok(isinstance(results[0], WebIDL.IDLInterface),
                   "%s - Should be an IDLInterface" % (prefix))
        # Make a copy, since we plan to modify it
        expectedMembers = list(expectedMembers)
        for m in results[0].members:
            name = m.identifier.name
            if (name, type(m)) in expectedMembers:
                harness.ok(True, "%s - %s - Should be a %s" % (prefix, name,
                                                               type(m)))
                expectedMembers.remove((name, type(m)))
            else:
                harness.ok(False, "%s - %s - Unknown symbol of type %s" %
                           (prefix, name, type(m)))
        # A bit of a hoop because we can't generate the error string if we pass
        if len(expectedMembers) == 0:
            harness.ok(True, "Found all the members")
        else:
            harness.ok(False,
                       "Expected member not found: %s of type %s" %
                       (expectedMembers[0][0], expectedMembers[0][1]))
        return results

    def shouldFail(prefix, iface):
        try:
            p = parser.reset()
            p.parse(iface)
            p.finish()
            harness.ok(False,
                       prefix + " - Interface passed when should've failed")
        except WebIDL.WebIDLError, e:
            harness.ok(True,
                       prefix + " - Interface failed as expected")
        except Exception, e:
            harness.ok(False,
                       prefix + " - Interface failed but not as a WebIDLError exception: %s" % e)

    iterableMembers = [(x, WebIDL.IDLMethod) for x in ["entries", "keys",
                                                       "values", "forEach"]]
    setROMembers = ([(x, WebIDL.IDLMethod) for x in ["has"]] +
                    [("__setlike", WebIDL.IDLMaplikeOrSetlike)] +
                    iterableMembers)
    setROMembers.extend([("size", WebIDL.IDLAttribute)])
    setRWMembers = ([(x, WebIDL.IDLMethod) for x in ["add",
                                                     "clear",
                                                     "delete"]] +
                    setROMembers)
    setROChromeMembers = ([(x, WebIDL.IDLMethod) for x in ["__add",
                                                           "__clear",
                                                           "__delete"]] +
                          setROMembers)
    setRWChromeMembers = ([(x, WebIDL.IDLMethod) for x in ["__add",
                                                           "__clear",
                                                           "__delete"]] +
                          setRWMembers)
    mapROMembers = ([(x, WebIDL.IDLMethod) for x in ["get", "has"]] +
                    [("__maplike", WebIDL.IDLMaplikeOrSetlike)] +
                    iterableMembers)
    mapROMembers.extend([("size", WebIDL.IDLAttribute)])
    mapRWMembers = ([(x, WebIDL.IDLMethod) for x in ["set",
                                                     "clear",
                                                     "delete"]] + mapROMembers)
    mapRWChromeMembers = ([(x, WebIDL.IDLMethod) for x in ["__set",
                                                           "__clear",
                                                           "__delete"]] +
                          mapRWMembers)

    # OK, now that we've used iterableMembers to set up the above, append
    # __iterable to it for the iterable<> case.
    iterableMembers.append(("__iterable", WebIDL.IDLIterable))

    valueIterableMembers = [("__iterable", WebIDL.IDLIterable)]
    valueIterableMembers.append(("__indexedgetter", WebIDL.IDLMethod))
    valueIterableMembers.append(("length", WebIDL.IDLAttribute))

    disallowedIterableNames = ["keys", "entries", "values"]
    disallowedMemberNames = ["forEach", "has", "size"] + disallowedIterableNames
    mapDisallowedMemberNames = ["get"] + disallowedMemberNames
    disallowedNonMethodNames = ["clear", "delete"]
    mapDisallowedNonMethodNames = ["set"] + disallowedNonMethodNames
    setDisallowedNonMethodNames = ["add"] + disallowedNonMethodNames
    unrelatedMembers = [("unrelatedAttribute", WebIDL.IDLAttribute),
                        ("unrelatedMethod", WebIDL.IDLMethod)]

    #
    # Simple Usage Tests
    #

    shouldPass("Iterable (key only)",
               """
               interface Foo1 {
               iterable<long>;
               readonly attribute unsigned long length;
               getter long(unsigned long index);
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, valueIterableMembers + unrelatedMembers)

    shouldPass("Iterable (key only) inheriting from parent",
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
               """, valueIterableMembers, numProductions=2)

    shouldPass("Iterable (key and value)",
               """
               interface Foo1 {
               iterable<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, iterableMembers + unrelatedMembers,
               # numProductions == 2 because of the generated iterator iface,
               numProductions=2)

    shouldPass("Iterable (key and value) inheriting from parent",
               """
               interface Foo1 : Foo2 {
               iterable<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, iterableMembers,
               # numProductions == 3 because of the generated iterator iface,
               numProductions=3)

    shouldPass("Maplike (readwrite)",
               """
               interface Foo1 {
               maplike<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, mapRWMembers + unrelatedMembers)

    shouldPass("Maplike (readwrite) inheriting from parent",
               """
               interface Foo1 : Foo2 {
               maplike<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, mapRWMembers, numProductions=2)

    shouldPass("Maplike (readwrite)",
               """
               interface Foo1 {
               maplike<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, mapRWMembers + unrelatedMembers)

    shouldPass("Maplike (readwrite) inheriting from parent",
               """
               interface Foo1 : Foo2 {
               maplike<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, mapRWMembers, numProductions=2)

    shouldPass("Maplike (readonly)",
               """
               interface Foo1 {
               readonly maplike<long, long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, mapROMembers + unrelatedMembers)

    shouldPass("Maplike (readonly) inheriting from parent",
               """
               interface Foo1 : Foo2 {
               readonly maplike<long, long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, mapROMembers, numProductions=2)

    shouldPass("Setlike (readwrite)",
               """
               interface Foo1 {
               setlike<long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, setRWMembers + unrelatedMembers)

    shouldPass("Setlike (readwrite) inheriting from parent",
               """
               interface Foo1 : Foo2 {
               setlike<long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, setRWMembers, numProductions=2)

    shouldPass("Setlike (readonly)",
               """
               interface Foo1 {
               readonly setlike<long>;
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, setROMembers + unrelatedMembers)

    shouldPass("Setlike (readonly) inheriting from parent",
               """
               interface Foo1 : Foo2 {
               readonly setlike<long>;
               };
               interface Foo2 {
               attribute long unrelatedAttribute;
               long unrelatedMethod();
               };
               """, setROMembers, numProductions=2)

    shouldPass("Inheritance of maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               """, mapRWMembers, numProductions=2)

    shouldPass("Implements with maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 {
               };
               Foo2 implements Foo1;
               """, mapRWMembers, numProductions=3)

    shouldPass("JS Implemented maplike interface",
               """
               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
               Constructor()]
               interface Foo1 {
               setlike<long>;
               };
               """, setRWChromeMembers)

    shouldPass("JS Implemented maplike interface",
               """
               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
               Constructor()]
               interface Foo1 {
               maplike<long, long>;
               };
               """, mapRWChromeMembers)

    #
    # Multiple maplike/setlike tests
    #

    shouldFail("Two maplike/setlikes on same interface",
               """
               interface Foo1 {
               setlike<long>;
               maplike<long, long>;
               };
               """)

    shouldFail("Two iterable/setlikes on same interface",
               """
               interface Foo1 {
               iterable<long>;
               maplike<long, long>;
               };
               """)

    shouldFail("Two iterables on same interface",
               """
               interface Foo1 {
               iterable<long>;
               iterable<long, long>;
               };
               """)

    shouldFail("Two maplike/setlikes in partials",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               partial interface Foo1 {
               setlike<long>;
               };
               """)

    shouldFail("Conflicting maplike/setlikes across inheritance",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               setlike<long>;
               };
               """)

    shouldFail("Conflicting maplike/iterable across inheritance",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               iterable<long>;
               };
               """)

    shouldFail("Conflicting maplike/setlikes across multistep inheritance",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               setlike<long>;
               };
               """)

    shouldFail("Consequential interface with conflicting maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 {
               setlike<long>;
               };
               Foo2 implements Foo1;
               """)

    shouldFail("Consequential interfaces with conflicting maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 {
               setlike<long>;
               };
               interface Foo3 {
               };
               Foo3 implements Foo1;
               Foo3 implements Foo2;
               """)

    #
    # Member name collision tests
    #

    def testConflictingMembers(likeMember, conflictName, expectedMembers, methodPasses):
        """
        Tests for maplike/setlike member generation against conflicting member
        names. If methodPasses is True, this means we expect the interface to
        pass in the case of method shadowing, and expectedMembers should be the
        list of interface members to check against on the passing interface.

        """
        if methodPasses:
            shouldPass("Conflicting method: %s and %s" % (likeMember, conflictName),
                       """
                       interface Foo1 {
                       %s;
                       [Throws]
                       void %s(long test1, double test2, double test3);
                       };
                       """ % (likeMember, conflictName), expectedMembers)
        else:
            shouldFail("Conflicting method: %s and %s" % (likeMember, conflictName),
                       """
                       interface Foo1 {
                       %s;
                       [Throws]
                       void %s(long test1, double test2, double test3);
                       };
                       """ % (likeMember, conflictName))
        # Inherited conflicting methods should ALWAYS fail
        shouldFail("Conflicting inherited method: %s and %s" % (likeMember, conflictName),
                   """
                   interface Foo1 {
                   void %s(long test1, double test2, double test3);
                   };
                   interface Foo2 : Foo1 {
                   %s;
                   };
                   """ % (conflictName, likeMember))
        shouldFail("Conflicting static method: %s and %s" % (likeMember, conflictName),
                   """
                   interface Foo1 {
                   %s;
                   static void %s(long test1, double test2, double test3);
                   };
                   """ % (likeMember, conflictName))
        shouldFail("Conflicting attribute: %s and %s" % (likeMember, conflictName),
                   """
                   interface Foo1 {
                   %s
                   attribute double %s;
                   };
                   """ % (likeMember, conflictName))
        shouldFail("Conflicting const: %s and %s" % (likeMember, conflictName),
                   """
                   interface Foo1 {
                   %s;
                   const double %s = 0;
                   };
                   """ % (likeMember, conflictName))
        shouldFail("Conflicting static attribute: %s and %s" % (likeMember, conflictName),
                   """
                   interface Foo1 {
                   %s;
                   static attribute long %s;
                   };
                   """ % (likeMember, conflictName))

    for member in disallowedIterableNames:
        testConflictingMembers("iterable<long, long>", member, iterableMembers, False)
    for member in mapDisallowedMemberNames:
        testConflictingMembers("maplike<long, long>", member, mapRWMembers, False)
    for member in disallowedMemberNames:
        testConflictingMembers("setlike<long>", member, setRWMembers, False)
    for member in mapDisallowedNonMethodNames:
        testConflictingMembers("maplike<long, long>", member, mapRWMembers, True)
    for member in setDisallowedNonMethodNames:
        testConflictingMembers("setlike<long>", member, setRWMembers, True)

    shouldPass("Inheritance of maplike/setlike with child member collision",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               void entries();
               };
               """, mapRWMembers, numProductions=2)

    shouldPass("Inheritance of multi-level maplike/setlike with child member collision",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               void entries();
               };
               """, mapRWMembers, numProductions=3)

    shouldFail("Interface with consequential maplike/setlike interface member collision",
               """
               interface Foo1 {
               void entries();
               };
               interface Foo2 {
               maplike<long, long>;
               };
               Foo1 implements Foo2;
               """)

    shouldFail("Maplike interface with consequential interface member collision",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 {
               void entries();
               };
               Foo1 implements Foo2;
               """)

    shouldPass("Consequential Maplike interface with inherited interface member collision",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 {
               void entries();
               };
               interface Foo3 : Foo2 {
               };
               Foo3 implements Foo1;
               """, mapRWMembers, numProductions=4)

    shouldPass("Inherited Maplike interface with consequential interface member collision",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 {
               void entries();
               };
               interface Foo3 : Foo1 {
               };
               Foo3 implements Foo2;
               """, mapRWMembers, numProductions=4)

    shouldFail("Inheritance of name collision with child maplike/setlike",
               """
               interface Foo1 {
               void entries();
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """)

    shouldFail("Inheritance of multi-level name collision with child maplike/setlike",
               """
               interface Foo1 {
               void entries();
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               maplike<long, long>;
               };
               """)

    shouldPass("Inheritance of attribute collision with parent maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               attribute double size;
               };
               """, mapRWMembers, numProductions=2)

    shouldPass("Inheritance of multi-level attribute collision with parent maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               attribute double size;
               };
               """, mapRWMembers, numProductions=3)

    shouldFail("Inheritance of attribute collision with child maplike/setlike",
               """
               interface Foo1 {
               attribute double size;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """)

    shouldFail("Inheritance of multi-level attribute collision with child maplike/setlike",
               """
               interface Foo1 {
               attribute double size;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               maplike<long, long>;
               };
               """)

    shouldFail("Inheritance of attribute/rw function collision with child maplike/setlike",
               """
               interface Foo1 {
               attribute double set;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """)

    shouldFail("Inheritance of const/rw function collision with child maplike/setlike",
               """
               interface Foo1 {
               const double set = 0;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """)

    shouldPass("Inheritance of rw function with same name in child maplike/setlike",
               """
               interface Foo1 {
               maplike<long, long>;
               };
               interface Foo2 : Foo1 {
               void clear();
               };
               """, mapRWMembers, numProductions=2)

    shouldFail("Inheritance of unforgeable attribute collision with child maplike/setlike",
               """
               interface Foo1 {
               [Unforgeable]
               attribute double size;
               };
               interface Foo2 : Foo1 {
               maplike<long, long>;
               };
               """)

    shouldFail("Inheritance of multi-level unforgeable attribute collision with child maplike/setlike",
               """
               interface Foo1 {
               [Unforgeable]
               attribute double size;
               };
               interface Foo2 : Foo1 {
               };
               interface Foo3 : Foo2 {
               maplike<long, long>;
               };
               """)

    shouldPass("Implemented interface with readonly allowable overrides",
               """
               interface Foo1 {
               readonly setlike<long>;
               readonly attribute boolean clear;
               };
               """, setROMembers + [("clear", WebIDL.IDLAttribute)])

    shouldPass("JS Implemented read-only interface with readonly allowable overrides",
               """
               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
               Constructor()]
               interface Foo1 {
               readonly setlike<long>;
               readonly attribute boolean clear;
               };
               """, setROChromeMembers + [("clear", WebIDL.IDLAttribute)])

    shouldFail("JS Implemented read-write interface with non-readwrite allowable overrides",
               """
               [JSImplementation="@mozilla.org/dom/test-interface-js-maplike;1",
               Constructor()]
               interface Foo1 {
               setlike<long>;
               readonly attribute boolean clear;
               };
               """)

    r = shouldPass("Check proper override of clear/delete/set",
                   """
                   interface Foo1 {
                   maplike<long, long>;
                   long clear(long a, long b, double c, double d);
                   long set(long a, long b, double c, double d);
                   long delete(long a, long b, double c, double d);
                   };
                   """, mapRWMembers)

    for m in r[0].members:
        if m.identifier.name in ["clear", "set", "delete"]:
            harness.ok(m.isMethod(), "%s should be a method" % m.identifier.name)
            harness.check(m.maxArgCount, 4, "%s should have 4 arguments" % m.identifier.name)
            harness.ok(not m.isMaplikeOrSetlikeOrIterableMethod(),
                       "%s should not be a maplike/setlike function" % m.identifier.name)
