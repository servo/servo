import WebIDL

def WebIDLTest(parser, harness):
    # Check that error messages put the '^' in the right place.

    threw = False
    input = 'interface ?'
    try:
        parser.parse(input)
        results = parser.finish()
    except WebIDL.WebIDLError, e:
        threw = True
        lines = str(e).split('\n')

        harness.check(len(lines), 3, 'Expected number of lines in error message')
        harness.check(lines[1], input, 'Second line shows error')
        harness.check(lines[2], ' ' * (len(input) - 1) + '^',
                      'Correct column pointer in error message')

    harness.ok(threw, "Should have thrown.")
