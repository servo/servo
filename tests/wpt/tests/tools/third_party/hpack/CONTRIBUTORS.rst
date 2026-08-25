Hyper is written and maintained by Cory Benfield and various contributors:

Development Lead
````````````````

- Cory Benfield <cory@lukasa.co.uk>

Contributors (hpack)
````````````````````
In chronological order:

- Sriram Ganesan (@elricL)

  - Implemented the Huffman encoding/decoding logic.

- Tatsuhiro Tsujikawa (@tatsuhiro-t)

  - Improved compression efficiency.

- Jim Carreer (@jimcarreer)

  - Support for 'never indexed' header fields.
  - Refactor of header table code.
  - Add support for returning bytestring headers instead of UTF-8 decoded ones.

- Eugene Obukhov (@irvind)

  - Improved decoding efficiency.

- Ian Foote (@Ian-Foote)

  - 25% performance improvement to integer decode.

- Davey Shafik (@dshafik)

  - More testing.

- Seth Michael Larson (@SethMichaelLarson)

  - Code cleanups.

- Bulat Khasanov (@KhasanovBI)

  - Performance improvement of static header search. Use dict search instead
    of linear search.

Contributors (hyper)
````````````````````

In chronological order:

- Alek Storm (@alekstorm)

  - Implemented Python 2.7 support.
  - Implemented HTTP/2 draft 10 support.
  - Implemented server push.

- Tetsuya Morimoto (@t2y)

  - Fixed a bug where large or incomplete frames were not handled correctly.
  - Added hyper command-line tool.
  - General code cleanups.

- Jerome De Cuyper (@jdecuyper)

  - Updated documentation and tests.

