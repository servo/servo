CSS tests have some particular requirements for metadata.

## Title element

``` html
<title>[Test Area]: [Title/Scope of Test]</title>
or
<title>[Test Area] Reference File</title>
```

The title appears in the generated index, so make sure it is
concise and descriptive. The role of the title is to
identify what specific detail of a feature or combination of
features is being tested, so that someone looking through an index
can see quickly what's tested in which file. In most cases, this
description should not require more than 5 or 6 words. There is no
need to provide the chapter or section in the title. For reference
file, the titles should not be specific to a test case as these
files may be used by multiple different tests.

Bad examples:

``` html
<title>CSS Test: Border Conflict Resolution</title>
<title>CSS Regions auto-height Reference</title>
```

Good examples:

``` html
<title>CSS Test: Border Conflict Resolution (width) - hidden/double
</title>
<title>CSS Reference File</title>
```

For CSS specifications other than CSS 2.1, you can include the
module name somewhere before the colon, like "CSS Selectors Test:"
or "CSS Test (Selectors):". Do not include the module version
number, since the test might get reused for the next version.

### Credits

``` html
<link rel="author" title="NAME_OF_AUTHOR"
href="[mailto:some@address or http://some.url]" />
```

Credits provide a way to identify the person or organization that
created the test and/or holds copyright in the test. This is useful
for reviewing purposes and for asking questions about the individual
test. A test can have multiple author credits if necessary.

Example 1:

``` html
<link rel="author" title="Boris Zbarsky"
href="mailto:bzbarsky@mit.edu" />
```

Example 2:

``` html
<link rel="author" title="Bert Bos"
href="http://www.w3.org/People/Bos/" />
```

Example 3:

``` html
<link rel="author" title="Microsoft" href="http://microsoft.com/" />
```

### Reviewer

``` html
<link rel="reviewer" title="NAME_OF_REVIEWER" href="[mailto:some@address
or http://some.url]" /> <!-- YYYY-MM-DD -->
```

If a test has passed review, then the reviewer should note this by
adding his or her name as a reviewer, along with the date of the
review. A test can have multiple reviewers if necessary. A reviewer
must be a person, not an organization.

Example 1:

``` html
<link rel="reviewer" title="Boris Zbarsky"
href="mailto:bzbarsky@mit.edu" /> <!-- 2008-02-19 -->
```

Example 2:

``` html
<link rel="reviewer" title="Bert Bos"
href="http://www.w3.org/People/Bos/" /> <!-- 2005-05-03 -->
```

If a test would pass review with some (non-metadata) changes and the
reviewer chooses to make these changes, then the reviewer should add
his or her name as a reviewer-author, along with the date of the
review, when checking in those changes. This indicates that the
reviewer-author approves of the original author's test when taken
with these proposed changes, and that someone else (possibly the
original author) must review the changes. The test is fully reviewed
only when the latest reviewer did not also contribute changes to the
test at the time of the review.

Example of a fully-reviewed test:

``` html
<link rel="author" title="Bert Bos"
href="http://www.w3.org/People/Bos/" />
<link rel="reviewer author" title="Boris Zbarsky"
href="mailto:bzbarsky@mit.edu" /> <!-- 2008-02-19 -->
<link rel="reviewer" title="Bert Bos"
href="http://www.w3.org/People/Bos/" /> <!-- 2008-04-22 -->
```

This test was written by Bert Bos, then reviewed by Boris Zbarsky,
who made some corrections before deeming it acceptable. Those
corrections were then reviewed and accepted by Bert Bos.

### Specification Links

Specification Links

``` html
<link rel="help" href="RELEVANT_SPEC_SECTION" />
```

The specification link elements provide a way to align the test with
information in the specification being tested.

* Links should link to relevant sections within the specification
* Use the anchors from the specification's Table of Contents
* A test can have multiple specification links
  * Always list the primary section that is being tested as the
    first item in the list of specification links
  * Order the list from the most used/specific to least used/specific
  * There is no need to list common incidental features like the
    color green if it is being used to validate the test unless the
    case is specifically testing the color green
* If the test is part of multiple test suites, link to the relevant
  sections of each spec.

Example 1:

``` html
<link rel="help"
href="http://www.w3.org/TR/CSS21/text.html#alignment-prop" />
```

Example 2:

``` html
<link rel="help"
href="http://www.w3.org/TR/CSS21/text.html#alignment-prop" />
<link rel="help" href="http://www.w3.org/TR/CSS21/visudet.html#q7" />
<link rel="help"
href="http://www.w3.org/TR/CSS21/visudet.html#line-height" />
<link rel="help"
href="http://www.w3.org/TR/CSS21/colors.html#background-properties" />
```

### Reference Links

*Reftests only*

``` html
<link rel="match" href="RELATIVE_PATH_TO_REFERENCE_FILE" />
<link rel="mismatch" href="RELATIVE_PATH_TO_REFERENCE_FILE" />
```

The reference link elements are used in reftests and provide
the list of reference file(s) that the test should be compared to.

* ```match``` references must be files that render identically to
  the test, but use an alternate means to do so
* Multiple match references are used when the test can match any of
  the reference files
  * If a test requires multiple match references that all need to
    match (for example, to catch when a reference fails in the same
    way the test does), then chain the references together, i.e.:
    place reference links to the additional match references in the
    reference files. It is recommended that the chained reference
    files form a loop (e.g.: a → b → c → a) so that a test linking
    to any reference in the chain will find all the references.
* ```mismatch``` references are files that render differently than
  the test file. A test may have any number of mismatch references.
  The test is considered to fail if it renders the same as any of
  the mismatch references.
  * Note that reference files may themselves have mismatch
    references. In that case the reference file must not render the
    same as any of its mismatch references in order to be considered
    valid.  If a reference is considered invalid (by the fact of not
    matching any of its match references, or matching any of its
    mismatch references), then a test that refers to the reference
    will be considered to have failed.
* Reference files may be dedicated reference files, images, or other
  tests

Example 1:

``` html
<link rel="match" href="green-box-ref.xht" />
```

Example 2:

``` html
<link rel="match" href="green-box-ref.xht" />
<link rel="match" href="blue-box-ref.xht" />
<link rel="mismatch" href="red-box-notref.xht" />
<link rel="mismatch" href="red-box-notref.xht" />
```

### Requirement Flags

<table>
<tr>
  <th>Token</th>
  <th>Description</th>
</tr>
<tr>
  <td>ahem</td>
  <td>Test requires
  <a href="http://www.w3.org/Style/CSS/Test/Fonts/Ahem">Ahem font</a>
  </td>
</tr>
<tr>
  <td>animated</td>
  <td>Test is animated in final state. (Cannot be verified using
    reftests/screenshots.)</td>
</tr>
<tr>
  <td>asis</td>
  <td>The test has particular markup formatting requirements and
    cannot be re-serialized.</td>
</tr>
<tr>
  <td>combo</td>
  <td>Test, which must have an unsuffixed filename number, is
     strictly the union of all the suffixed tests with the same name
     and number. (See File name format, below.)</td>
</tr>
<tr>
  <td>dom</td>
  <td>Requires support for JavaScript and the Document Object Model (
    DOM)</td>
</tr>
<tr>
  <td>font</td>
  <td>Requires a specific font to be installed. (Details must be
    provided and/or the font linked to in the test description)</td>
</tr>
<tr>
  <td>history</td>
  <td>User agent session history is required. Testing :visited is a
    good example where this may be used.</td>
</tr>
<tr>
  <td>HTMLonly</td>
  <td>Test case is only valid for HTML</td>
</tr>
<tr>
  <td>http</td>
  <td>Requires HTTP headers</td>
</tr>
<tr>
  <td>image</td>
  <td>Requires support for bitmap graphics and the graphic to load
  </td>
</tr>
<tr>
  <td>interact</td>
  <td>Requires human interaction (such as for testing scrolling
    behavior)</td>
</tr>
<tr>
  <td>invalid</td>
  <td>Tests handling of invalid CSS. Note: This case contains CSS
     properties and syntax that may not validate.</td>
</tr>
<tr>
  <td>may</td>
  <td>Behavior tested is preferred but OPTIONAL.
  <a href="http://www.ietf.org/rfc/rfc2119.txt">[RFC2119]</a></td>
</tr>
<tr>
  <td>namespace</td>
  <td>Requires support for XML Namespaces</td>
</tr>
<tr>
  <td>nonHTML</td>
  <td>Test case is only valid for formats besides HTML (e.g. XHTML
    or arbitrary XML)</td>
</tr>
<tr>
  <td>paged</td>
  <td>Only valid for paged media</td>
</tr>
<tr>
  <td>scroll</td>
  <td>Only valid for continuous (scrolling) media</td>
</tr>
<tr>
  <td>should</td>
  <td>Behavior tested is RECOMMENDED, but not REQUIRED. <a
    href="http://www.ietf.org/rfc/rfc2119.txt">[RFC2119]</a></td>
</tr>
<tr>
  <td>speech</td>
  <td>Device supports audio output. Text-to-speech (TTS) engine
    installed</td>
</tr>
<tr>
  <td>svg</td>
  <td>Requires support for vector graphics (SVG)</td>
</tr>
<tr>
  <td>userstyle</td>
  <td>Requires a user style sheet to be set</td>
</tr>
<tr>
  <td>32bit</td>
  <td>Assumes a 32-bit integer as the minimum (-2147483648) or
    maximum (2147483647) value</td>
</tr>
<tr>
  <td>96dpi</td>
  <td>Assumes 96dpi display</td>
</tr>
</table>


Example 1 (one token applies):
``` html
<meta name="flags" content="invalid" />
```

Example 2 (multiple tokens apply):

``` html
<meta name="flags" content="ahem image scroll" />
```

Example 3 (no tokens apply):

``` html
<meta name="flags" content="" />
```

### Test Assertions

``` html
<meta name="assert" content="TEST ASSERTION" />
```

This element should contain a complete detailed statement expressing
what specifically the test is attempting to prove. If the assertion
is only valid in certain cases, those conditions should be described
in the statement.

The assertion should not be:

* A copy of the title text
* A copy of the test verification instructions
* A duplicate of another assertion in the test suite
* A line or reference from the CSS specification unless that line is
  a complete assertion when taken out of context.

The test assertion is **optional**. It helps the reviewer understand
the goal of the test so that he or she can make sure it is being
tested correctly. Also, in case a problem is found with the test
later, the testing method (e.g. using `color` to determine pass/fail)
can be changed (e.g. to using `background-color`) while preserving
the intent of the test (e.g. testing support for ID selectors).

Examples of good test assertions:

* "This test checks that a background image with no intrinsic size
   covers the entire padding box."
* "This test checks that 'word-spacing' affects each space (U+0020)
  and non-breaking space (U+00A0)."
* "This test checks that if 'top' and 'bottom' offsets are specified
  on an absolutely-positioned replaced element, then any remaining
  space is split amongst the 'auto' vertical margins."
* "This test checks that 'text-indent' affects only the first line
  of a block container if that line is also the first formatted line
  of an element."
