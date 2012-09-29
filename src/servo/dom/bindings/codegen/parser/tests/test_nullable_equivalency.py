import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestNullableEquivalency1 {
          attribute long  a;
          attribute long? b;
        };

        interface TestNullableEquivalency2 {
          attribute ArrayBuffer  a;
          attribute ArrayBuffer? b;
        };

        /* Can't have dictionary-valued attributes, so can't test that here */

        enum TestNullableEquivalency4Enum {
          "Foo",
          "Bar"
        };

        interface TestNullableEquivalency4 {
          attribute TestNullableEquivalency4Enum  a;
          attribute TestNullableEquivalency4Enum? b;
        };

        interface TestNullableEquivalency5 {
          attribute TestNullableEquivalency4  a;
          attribute TestNullableEquivalency4? b;
        };

        interface TestNullableEquivalency6 {
          attribute boolean  a;
          attribute boolean? b;
        };

        interface TestNullableEquivalency7 {
          attribute DOMString  a;
          attribute DOMString? b;
        };

        /* Not implemented. */
        /*interface TestNullableEquivalency8 {
          attribute float  a;
          attribute float? b;
        };*/

        interface TestNullableEquivalency8 {
          attribute double  a;
          attribute double? b;
        };

        interface TestNullableEquivalency9 {
          attribute object  a;
          attribute object? b;
        };

        interface TestNullableEquivalency10 {
          attribute double[]  a;
          attribute double[]? b;
        };

        interface TestNullableEquivalency11 {
          attribute TestNullableEquivalency9[]  a;
          attribute TestNullableEquivalency9[]? b;
        };
    """)

    for decl in parser.finish():
        if decl.isInterface():
            checkEquivalent(decl, harness)

def checkEquivalent(iface, harness):
    type1 = iface.members[0].type
    type2 = iface.members[1].type

    harness.check(type1.nullable(), False, 'attr1 should not be nullable')
    harness.check(type2.nullable(), True, 'attr2 should be nullable')

    # We don't know about type1, but type2, the nullable type, definitely
    # shouldn't be builtin.
    harness.check(type2.builtin, False, 'attr2 should not be builtin')

    # Ensure that all attributes of type2 match those in type1, except for:
    #  - names on an ignore list,
    #  - names beginning with '_',
    #  - functions which throw when called with no args, and
    #  - class-level non-callables ("static variables").
    #
    # Yes, this is an ugly, fragile hack.  But it finds bugs...
    for attr in dir(type1):
        if attr.startswith('_') or \
           attr in ['nullable', 'builtin', 'filename', 'location',
                    'inner', 'QName'] or \
           (hasattr(type(type1), attr) and not callable(getattr(type1, attr))):
            continue

        a1 = getattr(type1, attr)

        if callable(a1):
            try:
                v1 = a1()
            except:
                # Can't call a1 with no args, so skip this attriute.
                continue

            try:
                a2 = getattr(type2, attr)
            except:
                harness.ok(False, 'Missing %s attribute on type %s in %s' % (attr, type2, iface))
                continue

            if not callable(a2):
                harness.ok(False, "%s attribute on type %s in %s wasn't callable" % (attr, type2, iface))
                continue

            v2 = a2()
            harness.check(v2, v1, '%s method return value' % attr)
        else:
            try:
                a2 = getattr(type2, attr)
            except:
                harness.ok(False, 'Missing %s attribute on type %s in %s' % (attr, type2, iface))
                continue

            harness.check(a2, a1, '%s attribute should match' % attr)
