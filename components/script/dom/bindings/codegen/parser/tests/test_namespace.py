def WebIDLTest(parser, harness):
    parser.parse(
        """
        namespace MyNamespace {
          attribute any foo;
          any bar();
        };
        """)

    results = parser.finish()
    harness.check(len(results), 1, "Should have a thing.")
    harness.ok(results[0].isNamespace(), "Our thing should be a namespace");
    harness.check(len(results[0].members), 2,
                  "Should have two things in our namespace")
    harness.ok(results[0].members[0].isAttr(), "First member is attribute")
    harness.ok(results[0].members[0].isStatic(), "Attribute should be static")
    harness.ok(results[0].members[1].isMethod(), "Second member is method")
    harness.ok(results[0].members[1].isStatic(), "Operation should be static")

    parser = parser.reset()
    parser.parse(
        """
        namespace MyNamespace {
          attribute any foo;
        };
        partial namespace MyNamespace {
          any bar();
        };
        """)

    results = parser.finish()
    harness.check(len(results), 2, "Should have things.")
    harness.ok(results[0].isNamespace(), "Our thing should be a namespace");
    harness.check(len(results[0].members), 2,
                  "Should have two things in our namespace")
    harness.ok(results[0].members[0].isAttr(), "First member is attribute")
    harness.ok(results[0].members[0].isStatic(), "Attribute should be static");
    harness.ok(results[0].members[1].isMethod(), "Second member is method")
    harness.ok(results[0].members[1].isStatic(), "Operation should be static");

    parser = parser.reset()
    parser.parse(
        """
        partial namespace MyNamespace {
          any bar();
        };
        namespace MyNamespace {
          attribute any foo;
        };
        """)

    results = parser.finish()
    harness.check(len(results), 2, "Should have things.")
    harness.ok(results[1].isNamespace(), "Our thing should be a namespace");
    harness.check(len(results[1].members), 2,
                  "Should have two things in our namespace")
    harness.ok(results[1].members[0].isAttr(), "First member is attribute")
    harness.ok(results[1].members[0].isStatic(), "Attribute should be static");
    harness.ok(results[1].members[1].isMethod(), "Second member is method")
    harness.ok(results[1].members[1].isStatic(), "Operation should be static");

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            namespace MyNamespace {
              static attribute any foo;
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            namespace MyNamespace {
              static any bar();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            namespace MyNamespace {
              any bar();
            };

            interface MyNamespace {
              any baz();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface MyNamespace {
              any baz();
            };

            namespace MyNamespace {
              any bar();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            namespace MyNamespace {
              any baz();
            };

            namespace MyNamespace {
              any bar();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            partial namespace MyNamespace {
              any baz();
            };

            interface MyNamespace {
              any bar();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            namespace MyNamespace {
              any bar();
            };

            partial interface MyNamespace {
              any baz();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            partial interface MyNamespace {
              any baz();
            };

            namespace MyNamespace {
              any bar();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface MyNamespace {
              any bar();
            };

            partial namespace MyNamespace {
              any baz();
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")
