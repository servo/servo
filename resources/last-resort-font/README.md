# Last Resort Font

*Last Resort* is a special-purpose font that includes a collection of glyphs to represent types of Unicode characters. These glyphs are specifically designed to allow users to recognize that a code point is one of the following:

* In which particular block a Unicode character is encoded
* In the PUA (*Private Use Area*) for which no agreement exists
* Unassigned (reserved for future assignment)
* A noncharacter

## Downloading the fonts

The latest pre-built binaries of the *Last Resort* fonts, which correspond to [Unicode Version 16.0.0](https://www.unicode.org/versions/Unicode16.0.0/), can be easily downloaded from the [Latest Release](https://github.com/unicode-org/last-resort-font/releases/latest/). These fonts may be updated for future versions of the Unicode Standard as time and resources permit.

## Last Resort &amp; Last Resort High-Efficiency

This repository includes two versions of the *Last Resort* font: *Last Resort* and *Last Resort High-Efficiency*. Although both fonts can be installed at the same time—because they have different names—you are encouraged to download and install only the one that is expected to work in the environments that you use:

* The file *LastResort-Regular.ttf* is a font named *Last Resort*, and its 'cmap' table includes a [Format 12](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage) (*Segmented coverage*) subtable that is supported in virtually all modern environments. This font is 8MB and includes 5,372 glyphs. Download and install this font if you are unsure which one to use.

* The file *LastResortHE-Regular.ttf* is a font named *Last Resort High-Efficiency*, and its 'cmap' table includes the more efficient—for this type of font—[Format 13](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings) (*Many-to-one range mappings*) subtable that may not be supported in some environments, such as most Windows and Adobe apps. Therefore, this font, which is considerably smaller (500K) and with fewer glyphs (362), requires greater care when downloaded and installed.

Both fonts’ 'cmap' tables include a [Format 4](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-4-segment-mapping-to-delta-values) (*Segment mapping to delta values*) subtable, which is a Windows OS requirement. That of the *Last Resort High-Efficiency* font is a stub (aka empty) subtable.

## Description

The glyphs of the *Last Resort* fonts are used as the backup of “last resort” to any other font: if a font cannot represent any particular Unicode character, the appropriate “missing” glyph from the *Last Resort* fonts is displayed instead. This provides users with the ability to more easily discern what type of character it is, and provides a clue as to what type of font they would need to display the characters properly. For more information, see *The Unicode Standard*, [Section 5.3](https://unicode.org/versions/Unicode16.0.0/core-spec/chapter-5/#G7730), *Unknown and Missing Characters*.

Overall, there are a number of advantages to using the *Last Resort* fonts for unrepresentable characters:

* Operating systems are freed from the overhead of providing a full Unicode font.
* Users see something more meaningful than a black box or other geometric shape for unrepresentable characters.
* Users familiar with the scripts being represented with the *Last Resort* fonts will readily identify what type of font needs to be installed in order to properly display the text.
* Users unfamiliar with the missing scripts are shown easily-identified symbols rather than lengthy strings of unidentifiable characters.

Unicode blocks are illustrated by a representative glyph from the block, chosen to be as distinct as possible from glyphs of other blocks. A square surrounding frame provides a common, recognizable element, and embedded within the edge of this frame, only visible at large size, are a form of the block name and its code point range to aid in identification.

![Sinhala](./images/LRSinhala.gif)&nbsp;![Hiragana](./images/LRHiragana.gif)&nbsp;![EgyptianHieroglyphs](./images/LREgyptianHieroglyphs.gif)&nbsp;![TransportMapSymbol](./images/LRTransportMapSymbols.gif)

There are two particularly special types of glyphs in the fonts. One of the types represents any unassigned code point in an existing block. The other type represents the 66 noncharacter code points: FDD0..FDEF, FFFE..FFFF, 1FFFE..1FFFF, 2FFFE..2FFFF, 3FFFE..3FFFF, 4FFFE..4FFFF, 5FFFE..5FFFF, 6FFFE..6FFFF, 7FFFE..7FFFF, 8FFFE..8FFFF, 9FFFE..9FFFF, AFFFE..AFFFF, BFFFE..BFFFF, CFFFE..CFFFF, DFFFE..DFFFF, EFFFE..EFFFF, FFFFE..FFFFF, and 10FFFE..10FFFF.

![UndefinedBMP](./images/LRUndefinedBMP.gif)&nbsp;![UndefinedPlane3](./images/LRUndefinedPlane3.gif)&nbsp;![NoncharacterBMP1](./images/LRNoncharacterBMP1.gif)&nbsp;![NoncharacterBMP2](./images/LRNoncharacterBMP2.gif)

Example glyphs were chosen in a number of ways. For example, almost all of the Brahmic scripts show the initial consonant *ka*, such as ක for Sinhala. Latin uses the letter *A*, because it’s the first letter, and because in each Latin block there is a letter *A* that is easily distinguished. Greek and Cyrillic use their last letters, *Ω* and *Я*, respectively, due to their distinctiveness. Most other scripts use their initial character where distinctive.

The *Last Resort* glyphs were drawn by Apple Inc., Michael Everson of [Evertype](https://www.evertype.com/), and Unicode, Inc.

## Building the fonts from source

**NOTE**: *Building the fonts from source requires that [Python Version 3](https://www.python.org/) and [AFDKO](https://github.com/adobe-type-tools/afdko) (Adobe Font Development Kit for OpenType) be installed.*

To build the fonts from source, simply execute the `build.sh` file.

## Getting Involved

Although the *Last Resort Font* repository is considered to be stable with no guarantee that it will be updated, suggestions can be provided by submitting a [new issue](https://github.com/unicode-org/last-resort-font/issues/new).

### Copyright & Licenses

Copyright © 1998-2024 Unicode, Inc. Unicode and the Unicode Logo are registered trademarks of Unicode, Inc. in the United States and other countries.

A CLA is required to contribute to this project - please refer to the [CONTRIBUTING.md](https://github.com/unicode-org/.github/blob/main/.github/CONTRIBUTING.md) file (or start a Pull Request) for more information.

The contents of this repository are governed by the Unicode [Terms of Use](https://www.unicode.org/copyright.html) and are released under the [SIL Open Font License, Version 1.1](./LICENSE).
