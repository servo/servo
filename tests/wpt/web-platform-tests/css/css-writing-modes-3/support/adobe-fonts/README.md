CSS Orientation Test
====

Overview
----
CSS Orientation Test are special-purpose OpenType fonts. This open source project provides all of the source files
that were used to build these OpenType fonts by using the AFDKO *makeotf* tool.

Getting Involved
----
Send suggestions for changes to the CSS Orientation Test project maintainer, lunde@adobe.com, for consideration.

Building
====

Pre-built font binaries
----
The installable font resources (font binaries) are not part of the source files.
They are available at  https://github.com/adobe-fonts/css-orientation-test/
The latest version of the font binaries is 1.005 (April 4th 2015).


Requirements
----

For building binary font files from source, installation of the
[Adobe Font Development Kit for OpenType](http://www.adobe.com/devnet/opentype/afdko.html) (AFDKO)
is necessary. The AFDKO tools are widely used for font development today, and are part of most font editor applications.

Building the fonts
----

The key to building OpenType fonts is *makeotf*, which is part of AFDKO. Information and usage instructions can be found
by executing *makeotf -h*.

In this repository, all necessary files are in place for building the OpenType fonts. For example, build a binary OTF font
for the full-width version like this, which also includes a post-process for inserting a "stub" 'DSIG' table:

    % makeotf -f cidfont.ps -r -ch UnicodeAll-UTF32-H
    % sfntedit -a DSIG=DSIG.bin CSSFWOrientationTest.otf
    % sfntedit -f CSSFWOrientationTest.otf

That is all.
