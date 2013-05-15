import WebIDL

def WebIDLTest(parser, harness):
    # Check that error messages put the '^' in the right place.

    threw = False
    input = """\
// This is a comment.
interface Foo {
};

/* This is also a comment. */
interface ?"""
    try:
        parser.parse(input)
        results = parser.finish()
    except WebIDL.WebIDLError, e:
        threw = True
        lines = str(e).split('\n')

        harness.check(len(lines), 3, 'Expected number of lines in error message')
        harness.ok(lines[0].endswith('line 6:10'), 'First line of error should end with "line 6:10", but was "%s".' % lines[0])
        harness.check(lines[1], 'interface ?', 'Second line of error message is the line which caused the error.')
        harness.check(lines[2], ' ' * (len('interface ?') - 1) + '^',
                      'Correct column pointer in error message.')

    harness.ok(threw, "Should have thrown.")

