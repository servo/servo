def firstArgType(method):
    return method.signatures()[0][1][0].type

def WebIDLTest(parser, harness):
    parser.parse("""
      dictionary Dict {
      };
      callback interface Foo {
      };
      interface Bar {
        // Bit of a pain to get things that have dictionary types
        void passDict(optional Dict arg);
        void passFoo(Foo arg);
        void passNullableUnion((object? or DOMString) arg);
        void passNullable(Foo? arg);
      };
    """)
    results = parser.finish()

    iface = results[2]
    harness.ok(iface.isInterface(), "Should have interface")
    dictMethod = iface.members[0]
    ifaceMethod = iface.members[1]
    nullableUnionMethod = iface.members[2]
    nullableIfaceMethod = iface.members[3]

    dictType = firstArgType(dictMethod)
    ifaceType = firstArgType(ifaceMethod)

    harness.ok(dictType.isDictionary(), "Should have dictionary type");
    harness.ok(ifaceType.isInterface(), "Should have interface type");
    harness.ok(ifaceType.isCallbackInterface(), "Should have callback interface type");

    harness.ok(not dictType.isDistinguishableFrom(ifaceType),
               "Dictionary not distinguishable from callback interface")
    harness.ok(not ifaceType.isDistinguishableFrom(dictType),
               "Callback interface not distinguishable from dictionary")

    nullableUnionType = firstArgType(nullableUnionMethod)
    nullableIfaceType = firstArgType(nullableIfaceMethod)

    harness.ok(nullableUnionType.isUnion(), "Should have union type");
    harness.ok(nullableIfaceType.isInterface(), "Should have interface type");
    harness.ok(nullableIfaceType.nullable(), "Should have nullable type");

    harness.ok(not nullableUnionType.isDistinguishableFrom(nullableIfaceType),
               "Nullable type not distinguishable from union with nullable "
               "member type")
    harness.ok(not nullableIfaceType.isDistinguishableFrom(nullableUnionType),
               "Union with nullable member type not distinguishable from "
               "nullable type")

    parser = parser.reset()
    parser.parse("""
      interface TestIface {
        void passKid(Kid arg);
        void passParent(Parent arg);
        void passGrandparent(Grandparent arg);
        void passImplemented(Implemented arg);
        void passImplementedParent(ImplementedParent arg);
        void passUnrelated1(Unrelated1 arg);
        void passUnrelated2(Unrelated2 arg);
        void passArrayBuffer(ArrayBuffer arg);
        void passArrayBuffer(ArrayBufferView arg);
      };

      interface Kid : Parent {};
      interface Parent : Grandparent {};
      interface Grandparent {};
      interface Implemented : ImplementedParent {};
      Parent implements Implemented;
      interface ImplementedParent {};
      interface Unrelated1 {};
      interface Unrelated2 {};
    """)
    results = parser.finish()

    iface = results[0]
    harness.ok(iface.isInterface(), "Should have interface")
    argTypes = [firstArgType(method) for method in iface.members]
    unrelatedTypes = [firstArgType(method) for method in iface.members[-3:]]

    for type1 in argTypes:
        for type2 in argTypes:
            distinguishable = (type1 is not type2 and
                               (type1 in unrelatedTypes or
                                type2 in unrelatedTypes))

            harness.check(type1.isDistinguishableFrom(type2),
                          distinguishable,
                          "Type %s should %sbe distinguishable from type %s" %
                          (type1, "" if distinguishable else "not ", type2))
            harness.check(type2.isDistinguishableFrom(type1),
                          distinguishable,
                          "Type %s should %sbe distinguishable from type %s" %
                          (type2, "" if distinguishable else "not ", type1))

    parser = parser.reset()
    parser.parse("""
      interface Dummy {};
      interface TestIface {
        void method(long arg1, TestIface arg2);
        void method(long arg1, long arg2);
        void method(long arg1, Dummy arg2);
        void method(DOMString arg1, DOMString arg2, DOMString arg3);
      };
    """)
    results = parser.finish()
    harness.check(len(results[1].members), 1,
                  "Should look like we have one method")
    harness.check(len(results[1].members[0].signatures()), 4,
                  "Should have four signatures")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Dummy {};
          interface TestIface {
            void method(long arg1, TestIface arg2);
            void method(long arg1, long arg2);
            void method(any arg1,  Dummy arg2);
            void method(DOMString arg1, DOMString arg2, DOMString arg3);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should throw when args before the distinguishing arg are not "
               "all the same type")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Dummy {};
          interface TestIface {
            void method(long arg1, TestIface arg2);
            void method(long arg1, long arg2);
            void method(any arg1,  DOMString arg2);
            void method(DOMString arg1, DOMString arg2, DOMString arg3);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should throw when there is no distinguishing index")
