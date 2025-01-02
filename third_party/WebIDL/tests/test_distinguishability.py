import WebIDL


def firstArgType(method):
    return method.signatures()[0][1][0].type


def WebIDLTest(parser, harness):
    parser.parse(
        """
      // Give our dictionary a required member so we don't need to
      // mess with optional and default values.
      dictionary Dict {
        required long member;
      };
      callback interface Foo {
      };
      interface Bar {
        // Bit of a pain to get things that have dictionary types
        undefined passDict(Dict arg);
        undefined passFoo(Foo arg);
        undefined passNullableUnion((object? or DOMString) arg);
        undefined passNullable(Foo? arg);
      };
    """
    )
    results = parser.finish()

    iface = results[2]
    harness.ok(iface.isInterface(), "Should have interface")
    dictMethod = iface.members[0]
    ifaceMethod = iface.members[1]
    nullableUnionMethod = iface.members[2]
    nullableIfaceMethod = iface.members[3]

    dictType = firstArgType(dictMethod)
    ifaceType = firstArgType(ifaceMethod)

    harness.ok(dictType.isDictionary(), "Should have dictionary type")
    harness.ok(ifaceType.isInterface(), "Should have interface type")
    harness.ok(ifaceType.isCallbackInterface(), "Should have callback interface type")

    harness.ok(
        not dictType.isDistinguishableFrom(ifaceType),
        "Dictionary not distinguishable from callback interface",
    )
    harness.ok(
        not ifaceType.isDistinguishableFrom(dictType),
        "Callback interface not distinguishable from dictionary",
    )

    nullableUnionType = firstArgType(nullableUnionMethod)
    nullableIfaceType = firstArgType(nullableIfaceMethod)

    harness.ok(nullableUnionType.isUnion(), "Should have union type")
    harness.ok(nullableIfaceType.isInterface(), "Should have interface type")
    harness.ok(nullableIfaceType.nullable(), "Should have nullable type")

    harness.ok(
        not nullableUnionType.isDistinguishableFrom(nullableIfaceType),
        "Nullable type not distinguishable from union with nullable " "member type",
    )
    harness.ok(
        not nullableIfaceType.isDistinguishableFrom(nullableUnionType),
        "Union with nullable member type not distinguishable from " "nullable type",
    )

    parser = parser.reset()
    parser.parse(
        """
      interface TestIface {
        undefined passKid(Kid arg);
        undefined passParent(Parent arg);
        undefined passGrandparent(Grandparent arg);
        undefined passUnrelated1(Unrelated1 arg);
        undefined passUnrelated2(Unrelated2 arg);
        undefined passArrayBuffer(ArrayBuffer arg);
        undefined passArrayBuffer(ArrayBufferView arg);
      };

      interface Kid : Parent {};
      interface Parent : Grandparent {};
      interface Grandparent {};
      interface Unrelated1 {};
      interface Unrelated2 {};
    """
    )
    results = parser.finish()

    iface = results[0]
    harness.ok(iface.isInterface(), "Should have interface")
    argTypes = [firstArgType(method) for method in iface.members]
    unrelatedTypes = [firstArgType(method) for method in iface.members[-3:]]

    for type1 in argTypes:
        for type2 in argTypes:
            distinguishable = type1 is not type2 and (
                type1 in unrelatedTypes or type2 in unrelatedTypes
            )

            harness.check(
                type1.isDistinguishableFrom(type2),
                distinguishable,
                "Type %s should %sbe distinguishable from type %s"
                % (type1, "" if distinguishable else "not ", type2),
            )
            harness.check(
                type2.isDistinguishableFrom(type1),
                distinguishable,
                "Type %s should %sbe distinguishable from type %s"
                % (type2, "" if distinguishable else "not ", type1),
            )

    parser = parser.reset()
    parser.parse(
        """
      interface Dummy {};
      interface TestIface {
        undefined method(long arg1, TestIface arg2);
        undefined method(long arg1, long arg2);
        undefined method(long arg1, Dummy arg2);
        undefined method(DOMString arg1, DOMString arg2, DOMString arg3);
      };
    """
    )
    results = parser.finish()
    harness.check(len(results[1].members), 1, "Should look like we have one method")
    harness.check(
        len(results[1].members[0].signatures()), 4, "Should have four signatures"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface Dummy {};
          interface TestIface {
            undefined method(long arg1, TestIface arg2);
            undefined method(long arg1, long arg2);
            undefined method(any arg1,  Dummy arg2);
            undefined method(DOMString arg1, DOMString arg2, DOMString arg3);
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should throw when args before the distinguishing arg are not "
        "all the same type",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface Dummy {};
          interface TestIface {
            undefined method(long arg1, TestIface arg2);
            undefined method(long arg1, long arg2);
            undefined method(any arg1,  DOMString arg2);
            undefined method(DOMString arg1, DOMString arg2, DOMString arg3);
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should throw when there is no distinguishing index")

    # Now let's test our whole distinguishability table
    argTypes = [
        "long",
        "short",
        "long?",
        "short?",
        "boolean",
        "boolean?",
        "undefined",
        "undefined?",
        "DOMString",
        "ByteString",
        "UTF8String",
        "Enum",
        "Enum2",
        "Interface",
        "Interface?",
        "AncestorInterface",
        "UnrelatedInterface",
        "CallbackInterface",
        "CallbackInterface?",
        "CallbackInterface2",
        "object",
        "Callback",
        "Callback2",
        "Dict",
        "Dict2",
        "sequence<long>",
        "sequence<short>",
        "record<DOMString, object>",
        "record<USVString, Dict>",
        "record<ByteString, long>",
        "record<UTF8String, long>",
        "any",
        "Promise<any>",
        "Promise<any>?",
        "USVString",
        "JSString",
        "ArrayBuffer",
        "ArrayBufferView",
        "Uint8Array",
        "Uint16Array",
        "(long or Callback)",
        "(long or Dict)",
    ]

    # Try to categorize things a bit to keep list lengths down
    def allBut(list1, list2):
        return [
            a
            for a in list1
            if a not in list2
            and (a != "any" and a != "Promise<any>" and a != "Promise<any>?")
        ]

    unionsWithCallback = ["(long or Callback)"]
    unionsNoCallback = ["(long or Dict)"]
    unions = unionsWithCallback + unionsNoCallback
    numerics = ["long", "short", "long?", "short?"]
    booleans = ["boolean", "boolean?"]
    undefineds = ["undefined", "undefined?"]
    primitives = numerics + booleans
    nonNumerics = allBut(argTypes, numerics + unions)
    nonBooleans = allBut(argTypes, booleans)
    strings = [
        "DOMString",
        "ByteString",
        "Enum",
        "Enum2",
        "USVString",
        "JSString",
        "UTF8String",
    ]
    nonStrings = allBut(argTypes, strings)
    nonObjects = undefineds + primitives + strings
    bufferSourceTypes = ["ArrayBuffer", "ArrayBufferView", "Uint8Array", "Uint16Array"]
    interfaces = [
        "Interface",
        "Interface?",
        "AncestorInterface",
        "UnrelatedInterface",
    ] + bufferSourceTypes
    nullables = [
        "long?",
        "short?",
        "boolean?",
        "undefined?",
        "Interface?",
        "CallbackInterface?",
        "Dict",
        "Dict2",
        "Date?",
        "any",
        "Promise<any>?",
    ] + unionsNoCallback
    sequences = ["sequence<long>", "sequence<short>"]
    nonUserObjects = nonObjects + interfaces + sequences
    otherObjects = allBut(argTypes, nonUserObjects + ["object"])
    notRelatedInterfaces = (
        nonObjects
        + ["UnrelatedInterface"]
        + otherObjects
        + sequences
        + bufferSourceTypes
    )
    records = [
        "record<DOMString, object>",
        "record<USVString, Dict>",
        "record<ByteString, long>",
        "record<UTF8String, long>",
    ]  # JSString not supported in records
    dicts = ["Dict", "Dict2"]
    callbacks = ["Callback", "Callback2"]
    callbackInterfaces = [
        "CallbackInterface",
        "CallbackInterface?",
        "CallbackInterface2",
    ]
    dictionaryLike = dicts + callbackInterfaces + records + unionsNoCallback

    # Build a representation of the distinguishability table as a dict
    # of dicts, holding True values where needed, holes elsewhere.
    data = dict()
    for type in argTypes:
        data[type] = dict()

    def setDistinguishable(type, types):
        for other in types:
            data[type][other] = True

    setDistinguishable("long", nonNumerics)
    setDistinguishable("short", nonNumerics)
    setDistinguishable("long?", allBut(nonNumerics, nullables))
    setDistinguishable("short?", allBut(nonNumerics, nullables))
    setDistinguishable("boolean", nonBooleans)
    setDistinguishable("boolean?", allBut(nonBooleans, nullables))
    setDistinguishable("undefined", allBut(argTypes, undefineds + dictionaryLike))
    setDistinguishable(
        "undefined?", allBut(argTypes, undefineds + dictionaryLike + nullables)
    )
    setDistinguishable("DOMString", nonStrings)
    setDistinguishable("ByteString", nonStrings)
    setDistinguishable("UTF8String", nonStrings)
    setDistinguishable("USVString", nonStrings)
    setDistinguishable("JSString", nonStrings)
    setDistinguishable("Enum", nonStrings)
    setDistinguishable("Enum2", nonStrings)
    setDistinguishable("Interface", notRelatedInterfaces)
    setDistinguishable("Interface?", allBut(notRelatedInterfaces, nullables))
    setDistinguishable("AncestorInterface", notRelatedInterfaces)
    setDistinguishable(
        "UnrelatedInterface", allBut(argTypes, ["object", "UnrelatedInterface"])
    )
    setDistinguishable(
        "CallbackInterface",
        allBut(nonUserObjects + callbacks + unionsWithCallback, undefineds),
    )
    setDistinguishable(
        "CallbackInterface?",
        allBut(nonUserObjects + callbacks + unionsWithCallback, nullables + undefineds),
    )
    setDistinguishable(
        "CallbackInterface2",
        allBut(nonUserObjects + callbacks + unionsWithCallback, undefineds),
    )
    setDistinguishable("object", nonObjects)
    setDistinguishable(
        "Callback",
        nonUserObjects + unionsNoCallback + dicts + records + callbackInterfaces,
    )
    setDistinguishable(
        "Callback2",
        nonUserObjects + unionsNoCallback + dicts + records + callbackInterfaces,
    )
    setDistinguishable(
        "Dict",
        allBut(nonUserObjects + unionsWithCallback + callbacks, nullables + undefineds),
    )
    setDistinguishable(
        "Dict2",
        allBut(nonUserObjects + unionsWithCallback + callbacks, nullables + undefineds),
    )
    setDistinguishable(
        "sequence<long>",
        allBut(argTypes, sequences + ["object"]),
    )
    setDistinguishable(
        "sequence<short>",
        allBut(argTypes, sequences + ["object"]),
    )
    setDistinguishable(
        "record<DOMString, object>",
        allBut(nonUserObjects + unionsWithCallback + callbacks, undefineds),
    )
    setDistinguishable(
        "record<USVString, Dict>",
        allBut(nonUserObjects + unionsWithCallback + callbacks, undefineds),
    )
    # JSString not supported in records
    setDistinguishable(
        "record<ByteString, long>",
        allBut(nonUserObjects + unionsWithCallback + callbacks, undefineds),
    )
    setDistinguishable(
        "record<UTF8String, long>",
        allBut(nonUserObjects + unionsWithCallback + callbacks, undefineds),
    )
    setDistinguishable("any", [])
    setDistinguishable("Promise<any>", [])
    setDistinguishable("Promise<any>?", [])
    setDistinguishable("ArrayBuffer", allBut(argTypes, ["ArrayBuffer", "object"]))
    setDistinguishable(
        "ArrayBufferView",
        allBut(argTypes, ["ArrayBufferView", "Uint8Array", "Uint16Array", "object"]),
    )
    setDistinguishable(
        "Uint8Array", allBut(argTypes, ["ArrayBufferView", "Uint8Array", "object"])
    )
    setDistinguishable(
        "Uint16Array", allBut(argTypes, ["ArrayBufferView", "Uint16Array", "object"])
    )
    setDistinguishable(
        "(long or Callback)",
        allBut(nonUserObjects + dicts + records + callbackInterfaces, numerics),
    )
    setDistinguishable(
        "(long or Dict)",
        allBut(nonUserObjects + callbacks, numerics + nullables + undefineds),
    )

    def areDistinguishable(type1, type2):
        return data[type1].get(type2, False)

    def checkDistinguishability(parser, type1, type2):
        idlTemplate = """
          enum Enum { "a", "b" };
          enum Enum2 { "c", "d" };
          interface Interface : AncestorInterface {};
          interface AncestorInterface {};
          interface UnrelatedInterface {};
          callback interface CallbackInterface {};
          callback interface CallbackInterface2 {};
          callback Callback = any();
          callback Callback2 = long(short arg);
          [LegacyTreatNonObjectAsNull] callback LegacyCallback1 = any();
          // Give our dictionaries required members so we don't need to
          // mess with optional and default values.
          dictionary Dict { required long member; };
          dictionary Dict2 { required long member; };
          interface TestInterface {%s
          };
        """
        if type1 in undefineds or type2 in undefineds:
            methods = """
                (%s or %s) myMethod();""" % (
                type1,
                type2,
            )
        else:
            methodTemplate = """
                undefined myMethod(%s arg);"""
            methods = (methodTemplate % type1) + (methodTemplate % type2)
        idl = idlTemplate % methods

        parser = parser.reset()
        threw = False
        try:
            parser.parse(idl)
            parser.finish()
        except WebIDL.WebIDLError:
            threw = True

        if areDistinguishable(type1, type2):
            harness.ok(
                not threw,
                "Should not throw for '%s' and '%s' because they are distinguishable"
                % (type1, type2),
            )
        else:
            harness.ok(
                threw,
                "Should throw for '%s' and '%s' because they are not distinguishable"
                % (type1, type2),
            )

    # Enumerate over everything in both orders, since order matters in
    # terms of our implementation of distinguishability checks
    for type1 in argTypes:
        for type2 in argTypes:
            checkDistinguishability(parser, type1, type2)
