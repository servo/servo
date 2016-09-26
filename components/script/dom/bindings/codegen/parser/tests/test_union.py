import WebIDL
import itertools
import string

# We'd like to use itertools.chain but it's 2.6 or higher.
def chain(*iterables):
    # chain('ABC', 'DEF') --> A B C D E F
    for it in iterables:
        for element in it:
            yield element

# We'd like to use itertools.combinations but it's 2.6 or higher.
def combinations(iterable, r):
    # combinations('ABCD', 2) --> AB AC AD BC BD CD
    # combinations(range(4), 3) --> 012 013 023 123
    pool = tuple(iterable)
    n = len(pool)
    if r > n:
        return
    indices = range(r)
    yield tuple(pool[i] for i in indices)
    while True:
        for i in reversed(range(r)):
            if indices[i] != i + n - r:
                break
        else:
            return
        indices[i] += 1
        for j in range(i+1, r):
            indices[j] = indices[j-1] + 1
        yield tuple(pool[i] for i in indices)

# We'd like to use itertools.combinations_with_replacement but it's 2.7 or
# higher.
def combinations_with_replacement(iterable, r):
    # combinations_with_replacement('ABC', 2) --> AA AB AC BB BC CC
    pool = tuple(iterable)
    n = len(pool)
    if not n and r:
        return
    indices = [0] * r
    yield tuple(pool[i] for i in indices)
    while True:
        for i in reversed(range(r)):
            if indices[i] != n - 1:
                break
        else:
            return
        indices[i:] = [indices[i] + 1] * (r - i)
        yield tuple(pool[i] for i in indices)

def WebIDLTest(parser, harness):
    types = ["float",
             "double",
             "short",
             "unsigned short",
             "long",
             "unsigned long",
             "long long",
             "unsigned long long",
             "boolean",
             "byte",
             "octet",
             "DOMString",
             "ByteString",
             "USVString",
             #"sequence<float>",
             "object",
             "ArrayBuffer",
             #"Date",
             "TestInterface1",
             "TestInterface2"]

    testPre = """
        interface TestInterface1 {
        };
        interface TestInterface2 {
        };
        """

    interface = testPre + """
        interface PrepareForTest {
        """
    for (i, type) in enumerate(types):
        interface += string.Template("""
          readonly attribute ${type} attr${i};
        """).substitute(i=i, type=type)
    interface += """
        };
        """

    parser.parse(interface)
    results = parser.finish()

    iface = results[2]

    parser = parser.reset()

    def typesAreDistinguishable(t):
        return all(u[0].isDistinguishableFrom(u[1]) for u in combinations(t, 2))
    def typesAreNotDistinguishable(t):
        return any(not u[0].isDistinguishableFrom(u[1]) for u in combinations(t, 2))
    def unionTypeName(t):
        if len(t) > 2:
            t[0:2] = [unionTypeName(t[0:2])]
        return "(" + " or ".join(t) + ")"

    # typeCombinations is an iterable of tuples containing the name of the type
    # as a string and the parsed IDL type.
    def unionTypes(typeCombinations, predicate):
        for c in typeCombinations:
            if predicate(t[1] for t in c):
                yield unionTypeName([t[0] for t in c])

    # We limit invalid union types with a union member type to the subset of 3
    # types with one invalid combination.
    # typeCombinations is an iterable of tuples containing the name of the type
    # as a string and the parsed IDL type.
    def invalidUnionWithUnion(typeCombinations):
        for c in typeCombinations:
            if (typesAreNotDistinguishable((c[0][1], c[1][1])) and
                typesAreDistinguishable((c[1][1], c[2][1])) and
                typesAreDistinguishable((c[0][1], c[2][1]))):
                yield unionTypeName([t[0] for t in c])

    # Create a list of tuples containing the name of the type as a string and
    # the parsed IDL type.
    types = zip(types, (a.type for a in iface.members))

    validUnionTypes = chain(unionTypes(combinations(types, 2), typesAreDistinguishable),
                            unionTypes(combinations(types, 3), typesAreDistinguishable))
    invalidUnionTypes = chain(unionTypes(combinations_with_replacement(types, 2), typesAreNotDistinguishable),
                              invalidUnionWithUnion(combinations(types, 3)))
    interface = testPre + """
        interface TestUnion {
        """
    for (i, type) in enumerate(validUnionTypes):
        interface += string.Template("""
          void method${i}(${type} arg);
          ${type} returnMethod${i}();
          attribute ${type} attr${i};
          void optionalMethod${i}(${type}? arg);
        """).substitute(i=i, type=type)
    interface += """
        };
        """
    parser.parse(interface)
    results = parser.finish()

    parser = parser.reset()

    for invalid in invalidUnionTypes:
        interface = testPre + string.Template("""
            interface TestUnion {
              void method(${type} arg);
            };
        """).substitute(type=invalid)

        threw = False
        try:
            parser.parse(interface)
            results = parser.finish()
        except:
            threw = True

        harness.ok(threw, "Should have thrown.")

        parser = parser.reset()
