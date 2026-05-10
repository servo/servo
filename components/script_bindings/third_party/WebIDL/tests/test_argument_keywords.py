def WebIDLTest(parser, harness):
    parser.parse(
        """
    interface Foo {
      undefined foo(object constructor);
    };
    """
    )

    results = parser.finish()
    harness.check(len(results), 1, "Should have an interface")
    iface = results[0]
    harness.check(len(iface.members), 1, "Should have an operation")
    operation = iface.members[0]
    harness.check(len(operation.signatures()), 1, "Should have one signature")
    (retval, args) = operation.signatures()[0]
    harness.check(len(args), 1, "Should have an argument")
    harness.check(
        args[0].identifier.name,
        "constructor",
        "Should have an identifier named 'constructor'",
    )
