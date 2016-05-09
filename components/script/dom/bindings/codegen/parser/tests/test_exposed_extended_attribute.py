import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
      [PrimaryGlobal] interface Foo {};
      [Global=(Bar1,Bar2)] interface Bar {};
      [Global=Baz2] interface Baz {};

      [Exposed=(Foo,Bar1)]
      interface Iface {
        void method1();

        [Exposed=Bar1]
        readonly attribute any attr;
      };

      [Exposed=Foo]
      partial interface Iface {
        void method2();
      };
    """)

    results = parser.finish()

    harness.check(len(results), 5, "Should know about five things");
    iface = results[3]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should have an interface here");
    members = iface.members
    harness.check(len(members), 3, "Should have three members")

    harness.ok(members[0].exposureSet == set(["Foo", "Bar"]),
               "method1 should have the right exposure set")
    harness.ok(members[0]._exposureGlobalNames == set(["Foo", "Bar1"]),
               "method1 should have the right exposure global names")

    harness.ok(members[1].exposureSet == set(["Bar"]),
               "attr should have the right exposure set")
    harness.ok(members[1]._exposureGlobalNames == set(["Bar1"]),
               "attr should have the right exposure global names")

    harness.ok(members[2].exposureSet == set(["Foo"]),
               "method2 should have the right exposure set")
    harness.ok(members[2]._exposureGlobalNames == set(["Foo"]),
               "method2 should have the right exposure global names")

    harness.ok(iface.exposureSet == set(["Foo", "Bar"]),
               "Iface should have the right exposure set")
    harness.ok(iface._exposureGlobalNames == set(["Foo", "Bar1"]),
               "Iface should have the right exposure global names")

    parser = parser.reset()
    parser.parse("""
      [PrimaryGlobal] interface Foo {};
      [Global=(Bar1,Bar2)] interface Bar {};
      [Global=Baz2] interface Baz {};

      interface Iface2 {
        void method3();
      };
    """)
    results = parser.finish()

    harness.check(len(results), 4, "Should know about four things");
    iface = results[3]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should have an interface here");
    members = iface.members
    harness.check(len(members), 1, "Should have one member")

    harness.ok(members[0].exposureSet == set(["Foo"]),
               "method3 should have the right exposure set")
    harness.ok(members[0]._exposureGlobalNames == set(["Foo"]),
               "method3 should have the right exposure global names")

    harness.ok(iface.exposureSet == set(["Foo"]),
               "Iface2 should have the right exposure set")
    harness.ok(iface._exposureGlobalNames == set(["Foo"]),
               "Iface2 should have the right exposure global names")

    parser = parser.reset()
    parser.parse("""
      [PrimaryGlobal] interface Foo {};
      [Global=(Bar1,Bar2)] interface Bar {};
      [Global=Baz2] interface Baz {};

      [Exposed=Foo]
      interface Iface3 {
        void method4();
      };

      [Exposed=(Foo,Bar1)]
      interface Mixin {
        void method5();
      };

      Iface3 implements Mixin;
    """)
    results = parser.finish()
    harness.check(len(results), 6, "Should know about six things");
    iface = results[3]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should have an interface here");
    members = iface.members
    harness.check(len(members), 2, "Should have two members")

    harness.ok(members[0].exposureSet == set(["Foo"]),
               "method4 should have the right exposure set")
    harness.ok(members[0]._exposureGlobalNames == set(["Foo"]),
               "method4 should have the right exposure global names")

    harness.ok(members[1].exposureSet == set(["Foo", "Bar"]),
               "method5 should have the right exposure set")
    harness.ok(members[1]._exposureGlobalNames == set(["Foo", "Bar1"]),
               "method5 should have the right exposure global names")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [Exposed=Foo]
            interface Bar {
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown on invalid Exposed value on interface.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Bar {
              [Exposed=Foo]
              readonly attribute bool attr;
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown on invalid Exposed value on attribute.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Bar {
              [Exposed=Foo]
              void operation();
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown on invalid Exposed value on operation.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Bar {
              [Exposed=Foo]
              const long constant = 5;
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown on invalid Exposed value on constant.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [Global] interface Foo {};
            [Global] interface Bar {};

            [Exposed=Foo]
            interface Baz {
              [Exposed=Bar]
              void method();
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown on member exposed where its interface is not.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [Global] interface Foo {};
            [Global] interface Bar {};

            [Exposed=Foo]
            interface Baz {
              void method();
            };

            [Exposed=Bar]
            interface Mixin {};

            Baz implements Mixin;
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown on LHS of implements being exposed where RHS is not.")
