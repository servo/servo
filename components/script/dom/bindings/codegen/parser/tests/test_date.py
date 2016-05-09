def WebIDLTest(parser, harness):
    parser.parse("""
        interface WithDates {
          attribute Date foo;
          void bar(Date arg);
          void baz(sequence<Date> arg);
        };
    """)

    results = parser.finish()
    harness.ok(results[0].members[0].type.isDate(), "Should have Date")
    harness.ok(results[0].members[1].signatures()[0][1][0].type.isDate(),
               "Should have Date argument")
    harness.ok(not results[0].members[2].signatures()[0][1][0].type.isDate(),
               "Should have non-Date argument")
