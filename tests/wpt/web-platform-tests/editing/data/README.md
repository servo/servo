# editing/data/*.js Format #

In the interests of keeping file size down, the format of these
(machine-generated) data files is relatively concise.  Unfortunately, this
means they can appear slightly cryptic to the untrained eye:

       ["foo[bar]baz",
         [["stylewithcss","false"],["bold",""]],
         "foo<b>[bar]</b>baz",
         [true,true],
         {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

But never fear!  It's not actually so complicated (assuming you understand the
relevant APIs to begin with).  Each line has the following format, which we
will explain in due course:

       ["initial HTML",
         [["command1", "arg1"], ["command2", "arg2"]],
         "expected HTML",
         [expected retval from command1, expected retval from command2],
         {"command1":[expected original/final indeterm/state/value 1],
          "command2":[expected original/final indeterm/state/value 2]}],

## Line 1: Initial HTML ##

    -> ["foo[bar]baz",
         [["stylewithcss","false"],["bold",""]],
         "foo<b>[bar]</b>baz",
         [true,true],
         {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

When testing, first a contenteditable div's innerHTML is set to the value given
here.  Then the characters []{} are located and removed, and the selection is
set to where they used to be, as follows:

  * [ and ] indicate the left or right endpoint of the selection, if it's in
    a text node.
  * { and } indicate the left or right endpoint of the selection, if it's not
    in a text node.

Thus `<b>[foo]</b>` means the selection start and end are (foo, 0) and (foo,
3), while `<b>{foo}</b>` means they're (`<b>`, 0) and (`<b>`, 1).
`<b>[foo}</b>` and `<b>{foo]</b>` are also possible.  There is no way to
describe backwards selections (i.e., distinguish anchor/focus).

In cases where you want the selection in a place where it's not possible to
place text, like `<table><tbody>{<tr></tr>}</tbody></table>`, another format
exists using data-start and data-end attributes.  It's only used in a few
tests, so it is not documented here.

## Line 2: commands ##

       ["foo[bar]baz",
    ->   [["stylewithcss","false"],["bold",""]],
         "foo<b>[bar]</b>baz",
         [true,true],
         {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

After the innerHTML of the editing host is filled in, the commands given here
are run in order, like this:

    document.execCommand("stylewithcss", false, "false");
    document.execCommand("bold", false, "");

Most tests have only one command run.  The exceptions are:

  1. styleWithCSS.  Tests that involve formatting elements or styles are run
     twice, once with styleWithCSS on and once with it off.
  2. defaultParagraphSeparator.  Tests that involve `<p>`s or `<div>`s are run
     twice, once with defaultParagraphSeparator set to "div" and once "p".
  3. multitest.js tests interactions between different commands, so it contains
     arbitrary combinations of commands.

## Line 3: expected HTML ##

       ["foo[bar]baz",
         [["stylewithcss","false"],["bold",""]],
    ->   "foo<b>[bar]</b>baz",
         [true,true],
         {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

After the commands are run, we check that the innerHTML of the editing host
matches the expected HTML provided here.  As on line 1, the characters []{}
(and data-start/data-end attributes) have special meaning and are not really
expected to be in the HTML.  However, on this line they don't affect the test's
processing -- there are no tests of what the final selection is.

## Line 4: expected return values ##

       ["foo[bar]baz",
         [["stylewithcss","false"],["bold",""]],
         "foo<b>[bar]</b>baz",
    ->   [true,true],
         {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

execCommand() returns a boolean: true if all went well, false if not (e.g.,
invalid value).  This line says what value each execCommand() call from line 2
was supposed to return.  Usually they'll all be true, but for tests of
error-handling they'll sometimes be false.

## Line 5: expected indeterm/state/value ##

       ["foo[bar]baz",
         [["stylewithcss","false"],["bold",""]],
         "foo<b>[bar]</b>baz",
         [true,true],
    ->   {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

For each command that we're running, we check queryCommandIndeterm(),
queryCommandState(), and queryCommandValue() before we begin running any of our
commands, and again after we've finished the last one.  (We don't run these
checks in between commands.)  For each command, this line gives an array of six
expected values, in order:

  1. Indeterm before
  2. State before
  3. Value before
  4. Indeterm after
  5. State after
  6. Value after

You can remember this by keeping in mind that the three "before" values come
before the three "after" values, and each set of three values is in
alphabetical order (indeterm/state/value).

## Analysis of a real-world example ##

Let's look back at the example we started with and see what it means:

    ["foo[bar]baz",
      [["stylewithcss","false"],["bold",""]],
      "foo<b>[bar]</b>baz",
      [true,true],
      {"stylewithcss":[false,true,"",false,false,""],"bold":[false,false,"",false,true,""]}],

Line 1: Set the innerHTML of our editing host to `foobarbaz`, and set the
selection's start and end inside the resulting text node, selecting the letters
"bar".  (We actually first set the innerHTML to `foo[bar]baz` and remove the
brackets afterwards.)

Line 2: Execute the commands:

    document.execCommand("stylewithcss", false, "false");
    document.execCommand("bold", false, "");

Before doing this, we record the indeterm/state/value for both "stylewithcss"
and "bold", and afterwards, we record them again.  We also record the return
value of both execCommand() calls.

Line 3: Our new innerHTML should be `foo<b>bar</b>baz`.  The [ and ] say where
we would theoretically want the selection to be, but no actual test is run.

Line 4: Both execCommands we ran should return true.

Line 5: We expect the indeterm for styleWithCSS to be false both before and
after, and the value to be "" before and after -- since they always are for
this command.  The state for styleWithCSS should be true beforehand, because
that's the way the previous test left it -- the testing framework doesn't clear
these settings in between tests.  (Thus the first test of styleWithCSS on the
page also tests the default value of the state.)  But we set it to false, so
after the tests it should be false.

We expect the indeterm for bold to be false both before and after, because
before nothing is bold, and after everything is bold.  Value should be ""
before and after, because it always is for bold.  The state before should be
false, but after should have changed to true.
