Autobahn Testsuite
==================

General information and installation instructions are available at
https://github.com/crossbario/autobahn-testsuite.

To improve performance, you should compile the C extension first::

    $ python setup.py build_ext --inplace

Running the test suite
----------------------

All commands below must be run from the directory containing this file.

To test the server::

    $ PYTHONPATH=.. python test_server.py
    $ wstest -m fuzzingclient

To test the client::

    $ wstest -m fuzzingserver
    $ PYTHONPATH=.. python test_client.py

Run the first command in a shell. Run the second command in another shell.
It should take about ten minutes to complete — wstest is the bottleneck.
Then kill the first one with Ctrl-C.

The test client or server shouldn't display any exceptions. The results are
stored in reports/clients/index.html.

Note that the Autobahn software only supports Python 2, while ``websockets``
only supports Python 3; you need two different environments.

Conformance notes
-----------------

Some test cases are more strict than the RFC. Given the implementation of the
library and the test echo client or server, ``websockets`` gets a "Non-Strict"
in these cases.

In 3.2, 3.3, 4.1.3, 4.1.4, 4.2.3, 4.2.4, and 5.15 ``websockets`` notices the
protocol error and closes the connection before it has had a chance to echo
the previous frame.

In 6.4.3 and 6.4.4, even though it uses an incremental decoder, ``websockets``
doesn't notice the invalid utf-8 fast enough to get a "Strict" pass. These
tests are more strict than the RFC.

