# Import the WebIDL module, so we can do isinstance checks and whatnot
import WebIDL

def WebIDLTest(parser, harness):
    # Basic functionality
    threw = False
    try:
        parser.parse("""
            A implements B;
            interface B {
              attribute long x;
            };
            interface A {
              attribute long y;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(not threw, "Should not have thrown on implements statement "
               "before interfaces")
    harness.check(len(results), 3, "We have three statements")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface), "B is an interface")
    harness.check(len(results[1].members), 1, "B has one member")
    A = results[2]
    harness.ok(isinstance(A, WebIDL.IDLInterface), "A is an interface")
    harness.check(len(A.members), 2, "A has two members")
    harness.check(A.members[0].identifier.name, "y", "First member is 'y'")
    harness.check(A.members[1].identifier.name, "x", "Second member is 'x'")

    # Duplicated member names not allowed
    threw = False
    try:
        parser.parse("""
            C implements D;
            interface D {
              attribute long x;
            };
            interface C {
              attribute long x;
            };
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on implemented interface duplicating "
               "a name on base interface")

    # Same, but duplicated across implemented interfaces
    threw = False
    try:
        parser.parse("""
            E implements F;
            E implements G;
            interface F {
              attribute long x;
            };
            interface G {
              attribute long x;
            };
            interface E {};
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on implemented interfaces "
               "duplicating each other's member names")

    # Same, but duplicated across indirectly implemented interfaces
    threw = False
    try:
        parser.parse("""
            H implements I;
            H implements J;
            I implements K;
            interface K {
              attribute long x;
            };
            interface L {
              attribute long x;
            };
            interface I {};
            interface J : L {};
            interface H {};
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on indirectly implemented interfaces "
               "duplicating each other's member names")

    # Same, but duplicated across an implemented interface and its parent
    threw = False
    try:
        parser.parse("""
            M implements N;
            interface O {
              attribute long x;
            };
            interface N : O {
              attribute long x;
            };
            interface M {};
        """)
        parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on implemented interface and its "
               "ancestor duplicating member names")

    # Reset the parser so we can actually find things where we expect
    # them in the list
    parser = parser.reset()

    # Diamonds should be allowed
    threw = False
    try:
        parser.parse("""
            P implements Q;
            P implements R;
            Q implements S;
            R implements S;
            interface Q {};
            interface R {};
            interface S {
              attribute long x;
            };
            interface P {};
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(not threw, "Diamond inheritance is fine")
    harness.check(results[6].identifier.name, "S", "We should be looking at 'S'")
    harness.check(len(results[6].members), 1, "S should have one member")
    harness.check(results[6].members[0].identifier.name, "x",
                  "S's member should be 'x'")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestInterface {
            };
            callback interface TestCallbackInterface {
            };
            TestInterface implements TestCallbackInterface;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should not allow callback interfaces on the right-hand side "
               "of 'implements'")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestInterface {
            };
            callback interface TestCallbackInterface {
            };
            TestCallbackInterface implements TestInterface;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should not allow callback interfaces on the left-hand side of "
               "'implements'")
    
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestInterface {
            };
            dictionary Dict {
            };
            Dict implements TestInterface;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should not allow non-interfaces on the left-hand side "
               "of 'implements'")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestInterface {
            };
            dictionary Dict {
            };
            TestInterface implements Dict;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should not allow non-interfaces on the right-hand side "
               "of 'implements'")

