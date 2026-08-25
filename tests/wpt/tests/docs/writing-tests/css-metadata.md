# CSS Metadata

CSS tests have some additional metadata.

### Specification Links

Each test **requires** at least one link to specifications:

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
href="https://www.w3.org/TR/CSS21/text.html#alignment-prop" />
```

Example 2:

``` html
<link rel="help"
href="https://www.w3.org/TR/CSS21/text.html#alignment-prop" />
<link rel="help" href="https://www.w3.org/TR/CSS21/visudet.html#q7" />
<link rel="help"
href="https://www.w3.org/TR/CSS21/visudet.html#line-height" />
<link rel="help"
href="https://www.w3.org/TR/CSS21/colors.html#background-properties" />
```

### Requirement Flags

If a test has any of the following requirements, a meta element can be added
to include the corresponding flags (tokens):

<table>
<tr>
  <th>Token</th>
  <th>Description</th>
</tr>
<tr>
  <td>asis</td>
  <td>The test has particular markup formatting requirements and
    cannot be re-serialized.</td>
</tr>
<tr>
  <td>HTMLonly</td>
  <td>Test case is only valid for HTML</td>
</tr>
<tr>
  <td>invalid</td>
  <td>Tests handling of invalid CSS. Note: This case contains CSS
     properties and syntax that may not validate.</td>
</tr>
<tr>
  <td>may</td>
  <td>Behavior tested is preferred but OPTIONAL.
  <a href="https://www.ietf.org/rfc/rfc2119.txt">[RFC2119]</a></td>
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
    href="https://www.ietf.org/rfc/rfc2119.txt">[RFC2119]</a></td>
</tr>
</table>

The following flags are **deprecated** and should not be declared by new tests.
Tests which satisfy the described criteria should simply be designated as
"manual" using [the `-manual` file name flag](file-names).

<table>
<tr>
  <th>Token</th>
  <th>Description</th>
</tr>
<tr>
  <td>animated</td>
  <td>Test is animated in final state. (Cannot be verified using
    reftests/screenshots.)</td>
</tr>
<tr>
  <td>font</td>
  <td>Requires a specific font to be installed at the OS level. (A link to the
      font to be installed must be provided; this is not needed if only web
      fonts are used.)</td>
</tr>
<tr>
  <td>history</td>
  <td>User agent session history is required. Testing :visited is a
    good example where this may be used.</td>
</tr>
<tr>
  <td>interact</td>
  <td>Requires human interaction (such as for testing scrolling
    behavior)</td>
</tr>
<tr>
  <td>speech</td>
  <td>Device supports audio output. Text-to-speech (TTS) engine
    installed</td>
</tr>
<tr>
  <td>userstyle</td>
  <td>Requires a user style sheet to be set</td>
</tr>
</table>


Example 1 (one token applies):

``` html
<meta name="flags" content="invalid" />
```

Example 2 (multiple tokens apply):

``` html
<meta name="flags" content="asis HTMLonly may" />
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

The test assertion is **optional**, but is highly recommended.
It helps the reviewer understand
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
