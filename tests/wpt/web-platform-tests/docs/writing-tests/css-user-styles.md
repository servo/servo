# CSS User Stylesheets

Some test may require special user style sheets to be applied in order
for the case to be verified. In order for proper indications and
prerequisite to be displayed every user style sheet should contain the
following rules.

``` css
#user-stylesheet-indication
{
   /* Used by the harness to display an indication there is a user
   style sheet applied */
    display: block!important;
}
```

The rule ```#user-stylesheet-indication``` is to be used by any
harness running the test suite.

A harness should identify test that need a user style sheet by
looking at their flags meta tag. It then should display appropriate
messages indicating if a style sheet is applied or if a style sheet
should not be applied.

Harness style sheet rules:

``` css
.userstyle
{
    color: green;
    display: none;
}
.nouserstyle
{
    color: red;
    display: none;
}
```

Harness userstyle flag found:

``` html
<p id="user-stylesheet-indication" class="userstyle">A user style
sheet is applied.</p>
```

Harness userstyle flag NOT found:

``` html
<p id="user-stylesheet-indication" class="nouserstyle">A user style
sheet is applied.</p>
```

Within the test case it is recommended that the case itself indicate
the necessary user style sheet that is required.

Examples: (code for the [`cascade.css`][cascade-css] file)

``` css
#cascade /* ID name should match user style sheet file name */
{
    /* Used by the test to hide the prerequisite */
    display: none;
}
```

The rule ```#cascade``` in the example above is used by the test
page to hide the prerequisite text. The rule name should match the
user style sheet CSS file name in order to keep this orderly.

Examples: (code for [the `cascade-###.xht` files][cascade-xht])

``` html
<p id="cascade">
    PREREQUISITE: The <a href="support/cascade.css">
    "cascade.css"</a> file is enabled as the user agent's user style
    sheet.
</p>
```

The id value should match the user style sheet CSS file name and the
user style sheet rule that is used to hide this text when the style
sheet is properly applied.

Please flag test that require user style sheets with the userstyle
flag so people running the tests know that a user style sheet is
required.

[cascade-css]: https://github.com/w3c/csswg-test/blob/master/css21/cascade/support/cascade.css
[cascade-xht]: https://github.com/w3c/csswg-test/blob/master/css21/cascade/cascade-001.xht
