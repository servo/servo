# Import the WebIDL module, so we can do isinstance checks and whatnot
import WebIDL

def WebIDLTest(parser, harness):
    # Basic functionality
    parser.parse(
        """
        interface Iface {
          [NewObject] readonly attribute Iface attr;
          [NewObject] Iface method();
        };
        """)
    results = parser.finish()
    harness.ok(results, "Should not have thrown on basic [NewObject] usage")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Iface {
              [Pure, NewObject] readonly attribute Iface attr;
            };
            """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "[NewObject] attributes must depend on something")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Iface {
              [Pure, NewObject] Iface method();
            };
            """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "[NewObject] methods must depend on something")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Iface {
              [Cached, NewObject, Affects=Nothing] readonly attribute Iface attr;
            };
            """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "[NewObject] attributes must not be [Cached]")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Iface {
              [StoreInSlot, NewObject, Affects=Nothing] readonly attribute Iface attr;
            };
            """)
        results = parser.finish()
    except:
        threw = True
    harness.ok(threw, "[NewObject] attributes must not be [StoreInSlot]")
