def WebIDLTest(parser, harness):
    parser.parse("""
      dictionary Dict {
        any foo;
        [ChromeOnly] any bar;
      };
    """)
    results = parser.finish()
    harness.check(len(results), 1, "Should have a dictionary")
    members = results[0].members;
    harness.check(len(members), 2, "Should have two members")
    # Note that members are ordered lexicographically, so "bar" comes
    # before "foo".
    harness.ok(members[0].getExtendedAttribute("ChromeOnly"),
               "First member is not ChromeOnly")
    harness.ok(not members[1].getExtendedAttribute("ChromeOnly"),
               "Second member is ChromeOnly")

    parser = parser.reset()
    parser.parse("""
      dictionary Dict {
        any foo;
        any bar;
      };

      interface Iface {
        [Constant, Cached] readonly attribute Dict dict;
      };
    """)
    results = parser.finish()
    harness.check(len(results), 2, "Should have a dictionary and an interface")

    parser = parser.reset()
    exception = None
    try:
      parser.parse("""
        dictionary Dict {
          any foo;
          [ChromeOnly] any bar;
        };

        interface Iface {
          [Constant, Cached] readonly attribute Dict dict;
        };
      """)
      results = parser.finish()
    except Exception, exception:
        pass

    harness.ok(exception, "Should have thrown.")
    harness.check(exception.message,
                  "[Cached] and [StoreInSlot] must not be used on an attribute "
                  "whose type contains a [ChromeOnly] dictionary member",
                  "Should have thrown the right exception")

    parser = parser.reset()
    exception = None
    try:
      parser.parse("""
        dictionary ParentDict {
          [ChromeOnly] any bar;
        };

        dictionary Dict : ParentDict {
          any foo;
        };

        interface Iface {
          [Constant, Cached] readonly attribute Dict dict;
        };
      """)
      results = parser.finish()
    except Exception, exception:
        pass

    harness.ok(exception, "Should have thrown (2).")
    harness.check(exception.message,
                  "[Cached] and [StoreInSlot] must not be used on an attribute "
                  "whose type contains a [ChromeOnly] dictionary member",
                  "Should have thrown the right exception (2)")

    parser = parser.reset()
    exception = None
    try:
      parser.parse("""
        dictionary GrandParentDict {
          [ChromeOnly] any baz;
        };

        dictionary ParentDict : GrandParentDict {
          any bar;
        };

        dictionary Dict : ParentDict {
          any foo;
        };

        interface Iface {
          [Constant, Cached] readonly attribute Dict dict;
        };
      """)
      results = parser.finish()
    except Exception, exception:
        pass

    harness.ok(exception, "Should have thrown (3).")
    harness.check(exception.message,
                  "[Cached] and [StoreInSlot] must not be used on an attribute "
                  "whose type contains a [ChromeOnly] dictionary member",
                  "Should have thrown the right exception (3)")
