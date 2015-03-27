These are syntax tests for WebIDL parser. A correct parser should fail on the tests in the "invalid" directory, and should be able to collect the data necessary to generate the XML serialization for valid fragments collected in the "valid" directory.

(the said XML serialization is the one produced by widlproc https://github.com/dontcallmedom/widlproc from which these tests have been imported)
