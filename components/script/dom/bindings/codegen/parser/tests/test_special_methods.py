import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface SpecialMethods {
          getter long long (unsigned long index);
          setter long long (unsigned long index, long long value);
          creator long long (unsigned long index, long long value);
          getter boolean (DOMString name);
          setter boolean (DOMString name, boolean value);
          creator boolean (DOMString name, boolean value);
          deleter boolean (DOMString name);
          readonly attribute unsigned long length;
        };

        interface SpecialMethodsCombination {
          setter creator long long (unsigned long index, long long value);
          getter deleter boolean (DOMString name);
          setter creator boolean (DOMString name, boolean value);
        };
    """)

    results = parser.finish()

    def checkMethod(method, QName, name,
                    static=False, getter=False, setter=False, creator=False,
                    deleter=False, legacycaller=False, stringifier=False):
        harness.ok(isinstance(method, WebIDL.IDLMethod),
                   "Should be an IDLMethod")
        harness.check(method.identifier.QName(), QName, "Method has the right QName")
        harness.check(method.identifier.name, name, "Method has the right name")
        harness.check(method.isStatic(), static, "Method has the correct static value")
        harness.check(method.isGetter(), getter, "Method has the correct getter value")
        harness.check(method.isSetter(), setter, "Method has the correct setter value")
        harness.check(method.isCreator(), creator, "Method has the correct creator value")
        harness.check(method.isDeleter(), deleter, "Method has the correct deleter value")
        harness.check(method.isLegacycaller(), legacycaller, "Method has the correct legacycaller value")
        harness.check(method.isStringifier(), stringifier, "Method has the correct stringifier value")

    harness.check(len(results), 2, "Expect 2 interfaces")

    iface = results[0]
    harness.check(len(iface.members), 8, "Expect 8 members")

    checkMethod(iface.members[0], "::SpecialMethods::__indexedgetter", "__indexedgetter",
                getter=True)
    checkMethod(iface.members[1], "::SpecialMethods::__indexedsetter", "__indexedsetter",
                setter=True)
    checkMethod(iface.members[2], "::SpecialMethods::__indexedcreator", "__indexedcreator",
                creator=True)
    checkMethod(iface.members[3], "::SpecialMethods::__namedgetter", "__namedgetter",
                getter=True)
    checkMethod(iface.members[4], "::SpecialMethods::__namedsetter", "__namedsetter",
                setter=True)
    checkMethod(iface.members[5], "::SpecialMethods::__namedcreator", "__namedcreator",
                creator=True)
    checkMethod(iface.members[6], "::SpecialMethods::__nameddeleter", "__nameddeleter",
                deleter=True)

    iface = results[1]
    harness.check(len(iface.members), 3, "Expect 3 members")

    checkMethod(iface.members[0], "::SpecialMethodsCombination::__indexedsettercreator",
                "__indexedsettercreator", setter=True, creator=True)
    checkMethod(iface.members[1], "::SpecialMethodsCombination::__namedgetterdeleter",
                "__namedgetterdeleter", getter=True, deleter=True)
    checkMethod(iface.members[2], "::SpecialMethodsCombination::__namedsettercreator",
                "__namedsettercreator", setter=True, creator=True)

    parser = parser.reset();

    threw = False
    try:
        parser.parse(
            """
            interface IndexedDeleter {
              deleter void(unsigned long index);
            };
            """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "There are no indexed deleters")
        
        
