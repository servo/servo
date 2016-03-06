def WebIDLTest(parser, harness):
    parser.parse("""
      dictionary Dict2 : Dict1 {
        long child = 5;
        Dict1 aaandAnother;
      };
      dictionary Dict1 {
        long parent;
        double otherParent;
      };
    """)
    results = parser.finish()

    dict1 = results[1];
    dict2 = results[0];

    harness.check(len(dict1.members), 2, "Dict1 has two members")
    harness.check(len(dict2.members), 2, "Dict2 has four members")

    harness.check(dict1.members[0].identifier.name, "otherParent",
                  "'o' comes before 'p'")
    harness.check(dict1.members[1].identifier.name, "parent",
                  "'o' really comes before 'p'")
    harness.check(dict2.members[0].identifier.name, "aaandAnother",
                  "'a' comes before 'c'")
    harness.check(dict2.members[1].identifier.name, "child",
                  "'a' really comes before 'c'")

    # Now reset our parser
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          dictionary Dict {
            long prop = 5;
            long prop;
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow name duplication in a dictionary")

    # Now reset our parser again
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          dictionary Dict1 : Dict2 {
            long prop = 5;
          };
          dictionary Dict2 : Dict3 {
            long prop2;
          };
          dictionary Dict3 {
            double prop;
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow name duplication in a dictionary and "
               "its ancestor")

    # More reset
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Iface {};
          dictionary Dict : Iface {
            long prop;
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow non-dictionary parents for dictionaries")

    # Even more reset
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A : B {};
            dictionary B : A {};
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow cycles in dictionary inheritance chains")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
              [TreatNullAs=EmptyString] DOMString foo;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow [TreatNullAs] on dictionary members");

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(A arg);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Trailing dictionary arg must be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo((A or DOMString) arg);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Trailing union arg containing a dictionary must be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(A arg1, optional long arg2);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Dictionary arg followed by optional arg must be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(A arg1, optional long arg2, long arg3);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(not threw,
               "Dictionary arg followed by non-optional arg doesn't have to be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo((A or DOMString) arg1, optional long arg2);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Union arg containing dictionary followed by optional arg must "
               "be optional")

    parser = parser.reset()
    parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(A arg1, long arg2);
            };
        """)
    results = parser.finish()
    harness.ok(True, "Dictionary arg followed by required arg can be required")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(optional A? arg1);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Dictionary arg must not be nullable")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(optional (A or long)? arg1);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Dictionary arg must not be in a nullable union")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(optional (A or long?) arg1);
            };
        """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw,
               "Dictionary must not be in a union with a nullable type")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary A {
            };
            interface X {
              void doFoo(optional (long? or A) arg1);
            };
        """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw,
               "A nullable type must not be in a union with a dictionary")

    parser = parser.reset()
    parser.parse("""
        dictionary A {
        };
        interface X {
          A? doFoo();
        };
    """)
    results = parser.finish()
    harness.ok(True, "Dictionary return value can be nullable")

    parser = parser.reset()
    parser.parse("""
        dictionary A {
        };
        interface X {
          void doFoo(optional A arg);
        };
    """)
    results = parser.finish()
    harness.ok(True, "Dictionary arg should actually parse")

    parser = parser.reset()
    parser.parse("""
        dictionary A {
        };
        interface X {
          void doFoo(optional (A or DOMString) arg);
        };
    """)
    results = parser.finish()
    harness.ok(True, "Union arg containing a dictionary should actually parse")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              Foo foo;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo3 : Foo {
              short d;
            };

            dictionary Foo2 : Foo3 {
              boolean c;
            };

            dictionary Foo1 : Foo2 {
              long a;
            };

            dictionary Foo {
              Foo1 b;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be a Dictionary that "
                      "inherits from its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              (Foo or DOMString)[]? b;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be a Nullable type "
                      "whose inner type includes its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              (DOMString or Foo) b;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be a Union type, one of "
                      "whose member types includes its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              sequence<sequence<sequence<Foo>>> c;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be a Sequence type "
                      "whose element type includes its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              (DOMString or Foo)[] d;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be an Array type "
                      "whose element type includes its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              Foo1 b;
            };

            dictionary Foo3 {
              Foo d;
            };

            dictionary Foo2 : Foo3 {
              short c;
            };

            dictionary Foo1 : Foo2 {
              long a;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be a Dictionary, one of whose "
                      "members or inherited members has a type that includes "
                      "its Dictionary.")

    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
            };

            dictionary Bar {
              Foo? d;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Member type must not be a nullable dictionary")

    parser = parser.reset();
    parser.parse("""
        dictionary Foo {
          unrestricted float  urFloat = 0;
          unrestricted float  urFloat2 = 1.1;
          unrestricted float  urFloat3 = -1.1;
          unrestricted float? urFloat4 = null;
          unrestricted float  infUrFloat = Infinity;
          unrestricted float  negativeInfUrFloat = -Infinity;
          unrestricted float  nanUrFloat = NaN;

          unrestricted double  urDouble = 0;
          unrestricted double  urDouble2 = 1.1;
          unrestricted double  urDouble3 = -1.1;
          unrestricted double? urDouble4 = null;
          unrestricted double  infUrDouble = Infinity;
          unrestricted double  negativeInfUrDouble = -Infinity;
          unrestricted double  nanUrDouble = NaN;
        };
    """)
    results = parser.finish()
    harness.ok(True, "Parsing default values for unrestricted types succeeded.")

    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              double f = Infinity;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to Infinity")

    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              double f = -Infinity;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to -Infinity")

    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              double f = NaN;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to NaN")

    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              float f = Infinity;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to Infinity")


    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              float f = -Infinity;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to -Infinity")

    parser = parser.reset();
    threw = False
    try:
        parser.parse("""
            dictionary Foo {
              float f = NaN;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to NaN")
