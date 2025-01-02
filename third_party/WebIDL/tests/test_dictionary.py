import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
      dictionary Dict2 : Dict1 {
        long child = 5;
        Dict1 aaandAnother;
      };
      dictionary Dict1 {
        long parent;
        double otherParent;
      };
    """
    )
    results = parser.finish()

    dict1 = results[1]
    dict2 = results[0]

    harness.check(len(dict1.members), 2, "Dict1 has two members")
    harness.check(len(dict2.members), 2, "Dict2 has four members")

    harness.check(
        dict1.members[0].identifier.name, "otherParent", "'o' comes before 'p'"
    )
    harness.check(
        dict1.members[1].identifier.name, "parent", "'o' really comes before 'p'"
    )
    harness.check(
        dict2.members[0].identifier.name, "aaandAnother", "'a' comes before 'c'"
    )
    harness.check(
        dict2.members[1].identifier.name, "child", "'a' really comes before 'c'"
    )

    # Test partial dictionary.
    parser = parser.reset()
    parser.parse(
        """
      dictionary A {
        long c;
        long g;
      };
      partial dictionary A {
        long h;
        long d;
      };
    """
    )
    results = parser.finish()

    dict1 = results[0]
    harness.check(len(dict1.members), 4, "Dict1 has four members")
    harness.check(dict1.members[0].identifier.name, "c", "c should be first")
    harness.check(dict1.members[1].identifier.name, "d", "d should come after c")
    harness.check(dict1.members[2].identifier.name, "g", "g should come after d")
    harness.check(dict1.members[3].identifier.name, "h", "h should be last")

    # Now reset our parser
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          dictionary Dict {
            long prop = 5;
            long prop;
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow name duplication in a dictionary")

    # Test no name duplication across normal and partial dictionary.
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          dictionary A {
            long prop = 5;
          };
          partial dictionary A {
            long prop;
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should not allow name duplication across normal and partial dictionary"
    )

    # Now reset our parser again
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          dictionary Dict1 : Dict2 {
            long prop = 5;
          };
          dictionary Dict2 : Dict3 {
            long prop2;
          };
          dictionary Dict3 {
            double prop;
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should not allow name duplication in a dictionary and " "its ancestor"
    )

    # More reset
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface Iface {};
          dictionary Dict : Iface {
            long prop;
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow non-dictionary parents for dictionaries")

    # Even more reset
    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A : B {};
            dictionary B : A {};
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow cycles in dictionary inheritance chains")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
              [LegacyNullToEmptyString] DOMString foo;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should not allow [LegacyNullToEmptyString] on dictionary members"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(A arg);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Trailing dictionary arg must be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional A arg);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Trailing dictionary arg must have a default value")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo((A or DOMString) arg);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Trailing union arg containing a dictionary must be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional (A or DOMString) arg);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Trailing union arg containing a dictionary must have a default value"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(A arg1, optional long arg2);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Dictionary arg followed by optional arg must be optional")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional A arg1, optional long arg2);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Dictionary arg followed by optional arg must have default value")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(A arg1, optional long arg2, long arg3);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        not threw,
        "Dictionary arg followed by non-optional arg doesn't have to be optional",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo((A or DOMString) arg1, optional long arg2);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Union arg containing dictionary followed by optional arg must " "be optional",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional (A or DOMString) arg1, optional long arg2);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Union arg containing dictionary followed by optional arg must "
        "have a default value",
    )

    parser = parser.reset()
    parser.parse(
        """
            dictionary A {
            };
            interface X {
              undefined doFoo(A arg1, long arg2);
            };
        """
    )
    parser.finish()
    harness.ok(True, "Dictionary arg followed by required arg can be required")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional A? arg1 = {});
            };
        """
        )
        parser.finish()
    except Exception as x:
        threw = x

    harness.ok(threw, "Optional dictionary arg must not be nullable")
    harness.ok(
        "nullable" in str(threw),
        "Must have the expected exception for optional nullable dictionary arg",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
              required long x;
            };
            interface X {
              undefined doFoo(A? arg1);
            };
        """
        )
        parser.finish()
    except Exception as x:
        threw = x

    harness.ok(threw, "Required dictionary arg must not be nullable")
    harness.ok(
        "nullable" in str(threw),
        "Must have the expected exception for required nullable " "dictionary arg",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional (A or long)? arg1 = {});
            };
        """
        )
        parser.finish()
    except Exception as x:
        threw = x

    harness.ok(threw, "Dictionary arg must not be in an optional nullable union")
    harness.ok(
        "nullable" in str(threw),
        "Must have the expected exception for optional nullable union "
        "arg containing dictionary",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
              required long x;
            };
            interface X {
              undefined doFoo((A or long)? arg1);
            };
        """
        )
        parser.finish()
    except Exception as x:
        threw = x

    harness.ok(threw, "Dictionary arg must not be in a required nullable union")
    harness.ok(
        "nullable" in str(threw),
        "Must have the expected exception for required nullable union "
        "arg containing dictionary",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(sequence<A?> arg1);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(not threw, "Nullable union should be allowed in a sequence argument")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional (A or long?) arg1);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Dictionary must not be in a union with a nullable type")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary A {
            };
            interface X {
              undefined doFoo(optional (long? or A) arg1);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "A nullable type must not be in a union with a dictionary")

    parser = parser.reset()
    parser.parse(
        """
        dictionary A {
        };
        interface X {
          A? doFoo();
        };
    """
    )
    parser.finish()
    harness.ok(True, "Dictionary return value can be nullable")

    parser = parser.reset()
    parser.parse(
        """
        dictionary A {
        };
        interface X {
          undefined doFoo(optional A arg = {});
        };
    """
    )
    parser.finish()
    harness.ok(True, "Dictionary arg should actually parse")

    parser = parser.reset()
    parser.parse(
        """
        dictionary A {
        };
        interface X {
          undefined doFoo(optional (A or DOMString) arg = {});
        };
    """
    )
    parser.finish()
    harness.ok(True, "Union arg containing a dictionary should actually parse")

    parser = parser.reset()
    parser.parse(
        """
        dictionary A {
        };
        interface X {
          undefined doFoo(optional (A or DOMString) arg = "abc");
        };
    """
    )
    parser.finish()
    harness.ok(
        True,
        "Union arg containing a dictionary with string default should actually parse",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              Foo foo;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Member type must not be its Dictionary.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
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
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Member type must not be a Dictionary that " "inherits from its Dictionary.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              (Foo or DOMString)[]? b;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Member type must not be a Nullable type "
        "whose inner type includes its Dictionary.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              (DOMString or Foo) b;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Member type must not be a Union type, one of "
        "whose member types includes its Dictionary.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              sequence<sequence<sequence<Foo>>> c;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Member type must not be a Sequence type "
        "whose element type includes its Dictionary.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              (DOMString or Foo)[] d;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Member type must not be an Array type "
        "whose element type includes its Dictionary.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
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
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Member type must not be a Dictionary, one of whose "
        "members or inherited members has a type that includes "
        "its Dictionary.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
            };

            dictionary Bar {
              Foo? d;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Member type must not be a nullable dictionary")

    parser = parser.reset()
    parser.parse(
        """
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
    """
    )
    parser.finish()
    harness.ok(True, "Parsing default values for unrestricted types succeeded.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              double f = Infinity;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to Infinity")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              double f = -Infinity;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to -Infinity")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              double f = NaN;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to NaN")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              float f = Infinity;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to Infinity")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              float f = -Infinity;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to -Infinity")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              float f = NaN;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Only unrestricted values can be initialized to NaN")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            dictionary Foo {
              long module;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(not threw, "Should be able to use 'module' as a dictionary member name")
