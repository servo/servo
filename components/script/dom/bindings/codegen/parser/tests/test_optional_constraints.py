def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface OptionalConstraints1 {
              void foo(optional byte arg1, byte arg2);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(not threw,
               "Should not have thrown on non-optional argument following "
               "optional argument.")

    parser = parser.reset()
    parser.parse("""
        interface OptionalConstraints2 {
          void foo(optional byte arg1 = 1, optional byte arg2 = 2,
                   optional byte arg3, optional byte arg4 = 4,
                   optional byte arg5, optional byte arg6 = 9);
        };
    """)
    results = parser.finish()
    args = results[0].members[0].signatures()[0][1]
    harness.check(len(args), 6, "Should have 6 arguments")
    harness.check(args[5].defaultValue.value, 9,
                  "Should have correct default value")
