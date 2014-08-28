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
              [TreatUndefinedAs=EmptyString] DOMString foo;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow [TreatUndefinedAs] on dictionary members");

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
              void doFoo(A arg1, optional long arg2);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Dictionary arg followed by optional arg must be optional")

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
              void doFoo((A or long)? arg1);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Dictionary arg must not be in a nullable union")
