def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface DuplicateQualifiers1 {
              getter getter byte foo(unsigned long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface DuplicateQualifiers2 {
              setter setter byte foo(unsigned long index, byte value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface DuplicateQualifiers3 {
              creator creator byte foo(unsigned long index, byte value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface DuplicateQualifiers4 {
              deleter deleter byte foo(unsigned long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface DuplicateQualifiers5 {
              getter deleter getter byte foo(unsigned long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        results = parser.parse("""
            interface DuplicateQualifiers6 {
              creator setter creator byte foo(unsigned long index, byte value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
