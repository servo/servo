Hyper is written and maintained by Cory Benfield and various contributors:

Development Lead
````````````````

- Cory Benfield <cory@lukasa.co.uk>

Contributors
````````````

In chronological order:

- Sriram Ganesan (@elricL)

  - Implemented the Huffman encoding/decoding logic.

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

- Maximilian Hils (@mhils)

  - Added repr for frames.
  - Improved frame initialization code.
  - Added flag validation.

- Thomas Kriechbaumer (@Kriechi)

  - Improved initialization code.
  - Fixed bugs in frame initialization code.
  - Improved frame repr for frames with non-printable bodies.

- Davey Shafik (@dshafik)

  - Fixed Alt Svc frame stream association.

- Seth Michael Larson (@SethMichaelLarson)

  - Performance improvements to serialization and parsing.

- Fred Thomsen (@fredthomsen)

  - Support for memoryview in DataFrames.

