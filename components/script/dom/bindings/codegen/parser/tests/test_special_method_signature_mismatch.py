def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch1 {
              getter long long foo(long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch2 {
              getter void foo(unsigned long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch3 {
              getter boolean foo(unsigned long index, boolean extraArg);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch4 {
              getter boolean foo(unsigned long... index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch5 {
              getter boolean foo(optional unsigned long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch6 {
              getter boolean foo();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch7 {
              deleter long long foo(long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch9 {
              deleter boolean foo(unsigned long index, boolean extraArg);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch10 {
              deleter boolean foo(unsigned long... index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch11 {
              deleter boolean foo(optional unsigned long index);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch12 {
              deleter boolean foo();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch13 {
              setter long long foo(long index, long long value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch15 {
              setter boolean foo(unsigned long index, boolean value, long long extraArg);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch16 {
              setter boolean foo(unsigned long index, boolean... value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch17 {
              setter boolean foo(unsigned long index, optional boolean value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch18 {
              setter boolean foo();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch20 {
              creator long long foo(long index, long long value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch22 {
              creator boolean foo(unsigned long index, boolean value, long long extraArg);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch23 {
              creator boolean foo(unsigned long index, boolean... value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch24 {
              creator boolean foo(unsigned long index, optional boolean value);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodSignatureMismatch25 {
              creator boolean foo();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
