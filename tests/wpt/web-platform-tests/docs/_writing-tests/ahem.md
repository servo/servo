---
layout: page
title: The Ahem Font
order: 11
---

A font called [Ahem][ahem-readme] has been developed which consists of
some very well defined glyphs of precise sizes and shapes; it is
especially useful for testing font and text properties.

The font's em-square is exactly square. Its ascent and descent
combined is exactly the size of the em square; this means that the
font's extent is exactly the same as its line-height, meaning that it
can be exactly aligned with padding, borders, margins, and so
forth. Its alphabetic baseline is 0.2em above its bottom, and 0.8em
below its top.

The font has four glyphs:

* X (U+0058):  A square exactly 1em in height and width.
* p (U+0070):  A rectangle exactly 0.2em high, 1em wide, and aligned so
that its top is flush with the baseline.
* É (U+00C9):  A rectangle exactly 0.8em high, 1em wide, and aligned so
that its bottom is flush with the baseline.
* [space] (U+0020):  A transparent space exactly 1em high and wide.

Most other US-ASCII characters in the font have the same glyph as X.

## Usage
If the test uses the Ahem font, make sure its computed font-size is a
multiple of 5px, otherwise baseline alignment may be rendered
inconsistently. A minimum computed font-size of 20px is suggested.

An explicit (i.e., not `normal`) line-height should also always be
used, with the difference between the computed line-height and
font-size being divisible by 2. In the common case, having the same
value for both is desirable.

Other font properties should make sure they have their default values;
as such, the `font` shorthand should normally be used.

As a result, what is typically recommended is:


``` css
div {
  font: 25px/1 Ahem;
}
```

Some things to avoid:

``` css
div {
  font: 1em/1em Ahem;  /* computed font-size is typically 16px and potentially
                          affected by parent elements */
}

div {
  font: 20px Ahem;  /* computed line-height value is normal */
}

div {
  /* doesn't use font shorthand; font-weight and font-style are inherited */
  font-family: Ahem;
  font-size: 25px;
  line-height: 50px;  /* the difference between computed line-height and
                         computed font-size is not divisible by 2
                         (50 - 25 = 25; 25 / 2 = 12.5). */
}
```

## Installing Ahem

After [downloading][download-ahem] the font, installation instructions
vary between platforms:

On Windows, right-click the downloaded file in File Explorer/Windows
Explorer (depending on Windows version) and select "Install" from the
menu.

On macOS, open the downloaded file in Font Book (the default
application for font files) and then click install.

On Linux, copy the file to `~/.local/share/fonts` and then run
`fc-cache`.

[ahem-readme]: https://www.w3.org/Style/CSS/Test/Fonts/Ahem/README
[download-ahem]: https://github.com/web-platform-tests/wpt/raw/master/fonts/Ahem.ttf
