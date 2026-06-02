.. _how-to-manage-output:

Managing pytest's output
=========================

.. _how-to-modifying-python-tb-printing:

Modifying Python traceback printing
--------------------------------------------------

Examples for modifying traceback printing:

.. code-block:: bash

    pytest --showlocals     # show local variables in tracebacks
    pytest -l               # show local variables (shortcut)
    pytest --no-showlocals  # hide local variables (if addopts enables them)

    pytest --capture=fd  # default, capture at the file descriptor level
    pytest --capture=sys # capture at the sys level
    pytest --capture=no  # don't capture
    pytest -s            # don't capture (shortcut)
    pytest --capture=tee-sys # capture to logs but also output to sys level streams

    pytest --tb=auto    # (default) 'long' tracebacks for the first and last
                         # entry, but 'short' style for the other entries
    pytest --tb=long    # exhaustive, informative traceback formatting
    pytest --tb=short   # shorter traceback format
    pytest --tb=line    # only one line per failure
    pytest --tb=native  # Python standard library formatting
    pytest --tb=no      # no traceback at all

The ``--full-trace`` causes very long traces to be printed on error (longer
than ``--tb=long``). It also ensures that a stack trace is printed on
**KeyboardInterrupt** (Ctrl+C).
This is very useful if the tests are taking too long and you interrupt them
with Ctrl+C to find out where the tests are *hanging*. By default no output
will be shown (because KeyboardInterrupt is caught by pytest). By using this
option you make sure a trace is shown.


Verbosity
--------------------------------------------------

Examples for modifying printing verbosity:

.. code-block:: bash

    pytest --quiet          # quiet - less verbose - mode
    pytest -q               # quiet - less verbose - mode (shortcut)
    pytest -v               # increase verbosity, display individual test names
    pytest -vv              # more verbose, display more details from the test output
    pytest -vvv             # not a standard , but may be used for even more detail in certain setups

The ``-v`` flag controls the verbosity of pytest output in various aspects: test session progress, assertion
details when tests fail, fixtures details with ``--fixtures``, etc.

.. regendoc:wipe

Consider this simple file:

.. code-block:: python

    # content of test_verbosity_example.py
    def test_ok():
        pass


    def test_words_fail():
        fruits1 = ["banana", "apple", "grapes", "melon", "kiwi"]
        fruits2 = ["banana", "apple", "orange", "melon", "kiwi"]
        assert fruits1 == fruits2


    def test_numbers_fail():
        number_to_text1 = {str(x): x for x in range(5)}
        number_to_text2 = {str(x * 10): x * 10 for x in range(5)}
        assert number_to_text1 == number_to_text2


    def test_long_text_fail():
        long_text = "Lorem ipsum dolor sit amet " * 10
        assert "hello world" in long_text

Executing pytest normally gives us this output (we are skipping the header to focus on the rest):

.. code-block:: pytest

    $ pytest --no-header
    =========================== test session starts ============================
    collected 4 items

    test_verbosity_example.py .FFF                                       [100%]

    ================================= FAILURES =================================
    _____________________________ test_words_fail ______________________________

        def test_words_fail():
            fruits1 = ["banana", "apple", "grapes", "melon", "kiwi"]
            fruits2 = ["banana", "apple", "orange", "melon", "kiwi"]
    >       assert fruits1 == fruits2
    E       AssertionError: assert ['banana', 'a...elon', 'kiwi'] == ['banana', 'a...elon', 'kiwi']
    E
    E         At index 2 diff: 'grapes' != 'orange'
    E         Use -v to get more diff

    test_verbosity_example.py:8: AssertionError
    ____________________________ test_numbers_fail _____________________________

        def test_numbers_fail():
            number_to_text1 = {str(x): x for x in range(5)}
            number_to_text2 = {str(x * 10): x * 10 for x in range(5)}
    >       assert number_to_text1 == number_to_text2
    E       AssertionError: assert {'0': 0, '1':..., '3': 3, ...} == {'0': 0, '10'...'30': 30, ...}
    E
    E         Omitting 1 identical items, use -vv to show
    E         Left contains 4 more items:
    E         {'1': 1, '2': 2, '3': 3, '4': 4}
    E         Right contains 4 more items:
    E         {'10': 10, '20': 20, '30': 30, '40': 40}
    E         Use -v to get more diff

    test_verbosity_example.py:14: AssertionError
    ___________________________ test_long_text_fail ____________________________

        def test_long_text_fail():
            long_text = "Lorem ipsum dolor sit amet " * 10
    >       assert "hello world" in long_text
    E       AssertionError: assert 'hello world' in 'Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ips... sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet '

    test_verbosity_example.py:19: AssertionError
    ========================= short test summary info ==========================
    FAILED test_verbosity_example.py::test_words_fail - AssertionError: asser...
    FAILED test_verbosity_example.py::test_numbers_fail - AssertionError: ass...
    FAILED test_verbosity_example.py::test_long_text_fail - AssertionError: a...
    ======================= 3 failed, 1 passed in 0.12s ========================

Notice that:

* Each test inside the file is shown by a single character in the output: ``.`` for passing, ``F`` for failure.
* ``test_words_fail`` failed, and we are shown a short summary indicating the index 2 of the two lists differ.
* ``test_numbers_fail`` failed, and we are shown a summary of left/right differences on dictionary items. Identical items are omitted.
* ``test_long_text_fail`` failed, and the right hand side of the ``in`` statement is truncated using ``...```
  because it is longer than an internal threshold (240 characters currently).

Now we can increase pytest's verbosity:

.. code-block:: pytest

    $ pytest --no-header -v
    =========================== test session starts ============================
    collecting ... collected 4 items

    test_verbosity_example.py::test_ok PASSED                            [ 25%]
    test_verbosity_example.py::test_words_fail FAILED                    [ 50%]
    test_verbosity_example.py::test_numbers_fail FAILED                  [ 75%]
    test_verbosity_example.py::test_long_text_fail FAILED                [100%]

    ================================= FAILURES =================================
    _____________________________ test_words_fail ______________________________

        def test_words_fail():
            fruits1 = ["banana", "apple", "grapes", "melon", "kiwi"]
            fruits2 = ["banana", "apple", "orange", "melon", "kiwi"]
    >       assert fruits1 == fruits2
    E       AssertionError: assert ['banana', 'a...elon', 'kiwi'] == ['banana', 'a...elon', 'kiwi']
    E
    E         At index 2 diff: 'grapes' != 'orange'
    E
    E         Full diff:
    E           [
    E               'banana',
    E               'apple',...
    E
    E         ...Full output truncated (7 lines hidden), use '-vv' to show

    test_verbosity_example.py:8: AssertionError
    ____________________________ test_numbers_fail _____________________________

        def test_numbers_fail():
            number_to_text1 = {str(x): x for x in range(5)}
            number_to_text2 = {str(x * 10): x * 10 for x in range(5)}
    >       assert number_to_text1 == number_to_text2
    E       AssertionError: assert {'0': 0, '1':..., '3': 3, ...} == {'0': 0, '10'...'30': 30, ...}
    E
    E         Omitting 1 identical items, use -vv to show
    E         Left contains 4 more items:
    E         {'1': 1, '2': 2, '3': 3, '4': 4}
    E         Right contains 4 more items:
    E         {'10': 10, '20': 20, '30': 30, '40': 40}
    E         ...
    E
    E         ...Full output truncated (16 lines hidden), use '-vv' to show

    test_verbosity_example.py:14: AssertionError
    ___________________________ test_long_text_fail ____________________________

        def test_long_text_fail():
            long_text = "Lorem ipsum dolor sit amet " * 10
    >       assert "hello world" in long_text
    E       AssertionError: assert 'hello world' in 'Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet '

    test_verbosity_example.py:19: AssertionError
    ========================= short test summary info ==========================
    FAILED test_verbosity_example.py::test_words_fail - AssertionError: asser...
    FAILED test_verbosity_example.py::test_numbers_fail - AssertionError: ass...
    FAILED test_verbosity_example.py::test_long_text_fail - AssertionError: a...
    ======================= 3 failed, 1 passed in 0.12s ========================

Notice now that:

* Each test inside the file gets its own line in the output.
* ``test_words_fail`` now shows the two failing lists in full, in addition to which index differs.
* ``test_numbers_fail`` now shows a text diff of the two dictionaries, truncated.
* ``test_long_text_fail`` no longer truncates the right hand side of the ``in`` statement, because the internal
  threshold for truncation is larger now (2400 characters currently).

Now if we increase verbosity even more:

.. code-block:: pytest

    $ pytest --no-header -vv
    =========================== test session starts ============================
    collecting ... collected 4 items

    test_verbosity_example.py::test_ok PASSED                            [ 25%]
    test_verbosity_example.py::test_words_fail FAILED                    [ 50%]
    test_verbosity_example.py::test_numbers_fail FAILED                  [ 75%]
    test_verbosity_example.py::test_long_text_fail FAILED                [100%]

    ================================= FAILURES =================================
    _____________________________ test_words_fail ______________________________

        def test_words_fail():
            fruits1 = ["banana", "apple", "grapes", "melon", "kiwi"]
            fruits2 = ["banana", "apple", "orange", "melon", "kiwi"]
    >       assert fruits1 == fruits2
    E       AssertionError: assert ['banana', 'apple', 'grapes', 'melon', 'kiwi'] == ['banana', 'apple', 'orange', 'melon', 'kiwi']
    E
    E         At index 2 diff: 'grapes' != 'orange'
    E
    E         Full diff:
    E           [
    E               'banana',
    E               'apple',
    E         -     'orange',
    E         ?      ^  ^^
    E         +     'grapes',
    E         ?      ^  ^ +
    E               'melon',
    E               'kiwi',
    E           ]

    test_verbosity_example.py:8: AssertionError
    ____________________________ test_numbers_fail _____________________________

        def test_numbers_fail():
            number_to_text1 = {str(x): x for x in range(5)}
            number_to_text2 = {str(x * 10): x * 10 for x in range(5)}
    >       assert number_to_text1 == number_to_text2
    E       AssertionError: assert {'0': 0, '1': 1, '2': 2, '3': 3, '4': 4} == {'0': 0, '10': 10, '20': 20, '30': 30, '40': 40}
    E
    E         Common items:
    E         {'0': 0}
    E         Left contains 4 more items:
    E         {'1': 1, '2': 2, '3': 3, '4': 4}
    E         Right contains 4 more items:
    E         {'10': 10, '20': 20, '30': 30, '40': 40}
    E
    E         Full diff:
    E           {
    E               '0': 0,
    E         -     '10': 10,
    E         ?       -    -
    E         +     '1': 1,
    E         -     '20': 20,
    E         ?       -    -
    E         +     '2': 2,
    E         -     '30': 30,
    E         ?       -    -
    E         +     '3': 3,
    E         -     '40': 40,
    E         ?       -    -
    E         +     '4': 4,
    E           }

    test_verbosity_example.py:14: AssertionError
    ___________________________ test_long_text_fail ____________________________

        def test_long_text_fail():
            long_text = "Lorem ipsum dolor sit amet " * 10
    >       assert "hello world" in long_text
    E       AssertionError: assert 'hello world' in 'Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet '

    test_verbosity_example.py:19: AssertionError
    ========================= short test summary info ==========================
    FAILED test_verbosity_example.py::test_words_fail - AssertionError: assert ['banana', 'apple', 'grapes', 'melon', 'kiwi'] == ['banana', 'apple', 'orange', 'melon', 'kiwi']

      At index 2 diff: 'grapes' != 'orange'

      Full diff:
        [
            'banana',
            'apple',
      -     'orange',
      ?      ^  ^^
      +     'grapes',
      ?      ^  ^ +
            'melon',
            'kiwi',
        ]
    FAILED test_verbosity_example.py::test_numbers_fail - AssertionError: assert {'0': 0, '1': 1, '2': 2, '3': 3, '4': 4} == {'0': 0, '10': 10, '20': 20, '30': 30, '40': 40}

      Common items:
      {'0': 0}
      Left contains 4 more items:
      {'1': 1, '2': 2, '3': 3, '4': 4}
      Right contains 4 more items:
      {'10': 10, '20': 20, '30': 30, '40': 40}

      Full diff:
        {
            '0': 0,
      -     '10': 10,
      ?       -    -
      +     '1': 1,
      -     '20': 20,
      ?       -    -
      +     '2': 2,
      -     '30': 30,
      ?       -    -
      +     '3': 3,
      -     '40': 40,
      ?       -    -
      +     '4': 4,
        }
    FAILED test_verbosity_example.py::test_long_text_fail - AssertionError: assert 'hello world' in 'Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet Lorem ipsum dolor sit amet '
    ======================= 3 failed, 1 passed in 0.12s ========================

Notice now that:

* Each test inside the file gets its own line in the output.
* ``test_words_fail`` gives the same output as before in this case.
* ``test_numbers_fail`` now shows a full text diff of the two dictionaries.
* ``test_long_text_fail`` also doesn't truncate on the right hand side as before, but now pytest won't truncate any
  text at all, regardless of its size.

Those were examples of how verbosity affects normal test session output, but verbosity also is used in other
situations, for example you are shown even fixtures that start with ``_`` if you use ``pytest --fixtures -v``.

Using higher verbosity levels (``-vvv``, ``-vvvv``, ...) is supported, but has no effect in pytest itself at the moment,
however some plugins might make use of higher verbosity.

.. _`pytest.fine_grained_verbosity`:

Fine-grained verbosity
~~~~~~~~~~~~~~~~~~~~~~

In addition to specifying the application wide verbosity level, it is possible to control specific aspects independently.
This is done by setting a verbosity level in the configuration file for the specific aspect of the output.

:confval:`verbosity_assertions`: Controls how verbose the assertion output should be when pytest is executed. Running
``pytest --no-header`` with a value of ``2`` would have the same output as the previous example, but each test inside
the file is shown by a single character in the output.

:confval:`verbosity_test_cases`: Controls how verbose the test execution output should be when pytest is executed.
Running ``pytest --no-header`` with a value of ``2`` would have the same output as the first verbosity example, but each
test inside the file gets its own line in the output.

.. _`pytest.detailed_failed_tests_usage`:

Producing a detailed summary report
--------------------------------------------------

The ``-r`` flag can be used to display a "short test summary info" at the end of the test session,
making it easy in large test suites to get a clear picture of all failures, skips, xfails, etc.

It defaults to ``fE`` to list failures and errors.

.. regendoc:wipe

Example:

.. code-block:: python

    # content of test_example.py
    import pytest


    @pytest.fixture
    def error_fixture():
        assert 0


    def test_ok():
        print("ok")


    def test_fail():
        assert 0


    def test_error(error_fixture):
        pass


    def test_skip():
        pytest.skip("skipping this test")


    def test_xfail():
        pytest.xfail("xfailing this test")


    @pytest.mark.xfail(reason="always xfail")
    def test_xpass():
        pass


.. code-block:: pytest

    $ pytest -ra
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 6 items

    test_example.py .FEsxX                                               [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError
    ================================ XFAILURES =================================
    ________________________________ test_xfail ________________________________

        def test_xfail():
    >       pytest.xfail("xfailing this test")
    E       _pytest.outcomes.XFailed: xfailing this test

    test_example.py:26: XFailed
    ================================= XPASSES ==================================
    ========================= short test summary info ==========================
    SKIPPED [1] test_example.py:22: skipping this test
    XFAIL test_example.py::test_xfail - reason: xfailing this test
    XPASS test_example.py::test_xpass - always xfail
    ERROR test_example.py::test_error - assert 0
    FAILED test_example.py::test_fail - assert 0
    == 1 failed, 1 passed, 1 skipped, 1 xfailed, 1 xpassed, 1 error in 0.12s ===

The ``-r`` options accepts a number of characters after it, with ``a`` used
above meaning "all except passes".

Here is the full list of available characters that can be used:

 - ``f`` - failed
 - ``E`` - error
 - ``s`` - skipped
 - ``x`` - xfailed
 - ``X`` - xpassed
 - ``p`` - passed
 - ``P`` - passed with output

Special characters for (de)selection of groups:

 - ``a`` - all except ``pP``
 - ``A`` - all
 - ``N`` - none, this can be used to display nothing (since ``fE`` is the default)

More than one character can be used, so for example to only see failed and skipped tests, you can execute:

.. code-block:: pytest

    $ pytest -rfs
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 6 items

    test_example.py .FEsxX                                               [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError
    ========================= short test summary info ==========================
    FAILED test_example.py::test_fail - assert 0
    SKIPPED [1] test_example.py:22: skipping this test
    == 1 failed, 1 passed, 1 skipped, 1 xfailed, 1 xpassed, 1 error in 0.12s ===

Using ``p`` lists the passing tests, whilst ``P`` adds an extra section "PASSES" with those tests that passed but had
captured output:

.. code-block:: pytest

    $ pytest -rpP
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 6 items

    test_example.py .FEsxX                                               [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError
    ================================== PASSES ==================================
    _________________________________ test_ok __________________________________
    --------------------------- Captured stdout call ---------------------------
    ok
    ========================= short test summary info ==========================
    PASSED test_example.py::test_ok
    == 1 failed, 1 passed, 1 skipped, 1 xfailed, 1 xpassed, 1 error in 0.12s ===

Creating resultlog format files
--------------------------------------------------

To create plain-text machine-readable result files you can issue:

.. code-block:: bash

    pytest --resultlog=path

and look at the content at the ``path`` location.  Such files are used e.g.
by the `PyPy-test`_ web page to show test results over several revisions.

.. warning::

    This option is rarely used and is scheduled for removal in pytest 6.0.

    If you use this option, consider using the new `pytest-reportlog <https://github.com/pytest-dev/pytest-reportlog>`__ plugin instead.

    See :ref:`the deprecation docs <resultlog deprecated>` for more information.


.. _`PyPy-test`: http://buildbot.pypy.org/summary


Creating JUnitXML format files
----------------------------------------------------

To create result files which can be read by Jenkins_ or other Continuous
integration servers, use this invocation:

.. code-block:: bash

    pytest --junit-xml=path

to create an XML file at ``path``.



To set the name of the root test suite xml item, you can configure the ``junit_suite_name`` option in your config file:

.. code-block:: ini

    [pytest]
    junit_suite_name = my_suite

.. versionadded:: 4.0

JUnit XML specification seems to indicate that ``"time"`` attribute
should report total test execution times, including setup and teardown
(`1 <http://windyroad.com.au/dl/Open%20Source/JUnit.xsd>`_, `2
<https://www.ibm.com/support/knowledgecenter/en/SSQ2R2_14.1.0/com.ibm.rsar.analysis.codereview.cobol.doc/topics/cac_useresults_junit.html>`_).
It is the default pytest behavior. To report just call durations
instead, configure the ``junit_duration_report`` option like this:

.. code-block:: ini

    [pytest]
    junit_duration_report = call

.. _record_property example:

record_property
~~~~~~~~~~~~~~~~~

If you want to log additional information for a test, you can use the
``record_property`` fixture:

.. code-block:: python

    def test_function(record_property):
        record_property("example_key", 1)
        assert True

This will add an extra property ``example_key="1"`` to the generated
``testcase`` tag:

.. code-block:: xml

    <testcase classname="test_function" file="test_function.py" line="0" name="test_function" time="0.0009">
      <properties>
        <property name="example_key" value="1" />
      </properties>
    </testcase>

Alternatively, you can integrate this functionality with custom markers:

.. code-block:: python

    # content of conftest.py


    def pytest_collection_modifyitems(session, config, items):
        for item in items:
            for marker in item.iter_markers(name="test_id"):
                test_id = marker.args[0]
                item.user_properties.append(("test_id", test_id))

And in your tests:

.. code-block:: python

    # content of test_function.py
    import pytest


    @pytest.mark.test_id(1501)
    def test_function():
        assert True

Will result in:

.. code-block:: xml

    <testcase classname="test_function" file="test_function.py" line="0" name="test_function" time="0.0009">
      <properties>
        <property name="test_id" value="1501" />
      </properties>
    </testcase>

.. warning::

    Please note that using this feature will break schema verifications for the latest JUnitXML schema.
    This might be a problem when used with some CI servers.


record_xml_attribute
~~~~~~~~~~~~~~~~~~~~~~~

To add an additional xml attribute to a testcase element, you can use
``record_xml_attribute`` fixture. This can also be used to override existing values:

.. code-block:: python

    def test_function(record_xml_attribute):
        record_xml_attribute("assertions", "REQ-1234")
        record_xml_attribute("classname", "custom_classname")
        print("hello world")
        assert True

Unlike ``record_property``, this will not add a new child element.
Instead, this will add an attribute ``assertions="REQ-1234"`` inside the generated
``testcase`` tag and override the default ``classname`` with ``"classname=custom_classname"``:

.. code-block:: xml

    <testcase classname="custom_classname" file="test_function.py" line="0" name="test_function" time="0.003" assertions="REQ-1234">
        <system-out>
            hello world
        </system-out>
    </testcase>

.. warning::

    ``record_xml_attribute`` is an experimental feature, and its interface might be replaced
    by something more powerful and general in future versions. The
    functionality per-se will be kept, however.

    Using this over ``record_xml_property`` can help when using ci tools to parse the xml report.
    However, some parsers are quite strict about the elements and attributes that are allowed.
    Many tools use an xsd schema (like the example below) to validate incoming xml.
    Make sure you are using attribute names that are allowed by your parser.

    Below is the Scheme used by Jenkins to validate the XML report:

    .. code-block:: xml

        <xs:element name="testcase">
            <xs:complexType>
                <xs:sequence>
                    <xs:element ref="skipped" minOccurs="0" maxOccurs="1"/>
                    <xs:element ref="error" minOccurs="0" maxOccurs="unbounded"/>
                    <xs:element ref="failure" minOccurs="0" maxOccurs="unbounded"/>
                    <xs:element ref="system-out" minOccurs="0" maxOccurs="unbounded"/>
                    <xs:element ref="system-err" minOccurs="0" maxOccurs="unbounded"/>
                </xs:sequence>
                <xs:attribute name="name" type="xs:string" use="required"/>
                <xs:attribute name="assertions" type="xs:string" use="optional"/>
                <xs:attribute name="time" type="xs:string" use="optional"/>
                <xs:attribute name="classname" type="xs:string" use="optional"/>
                <xs:attribute name="status" type="xs:string" use="optional"/>
            </xs:complexType>
        </xs:element>

.. warning::

    Please note that using this feature will break schema verifications for the latest JUnitXML schema.
    This might be a problem when used with some CI servers.

.. _record_testsuite_property example:

record_testsuite_property
^^^^^^^^^^^^^^^^^^^^^^^^^

.. versionadded:: 4.5

If you want to add a properties node at the test-suite level, which may contains properties
that are relevant to all tests, you can use the ``record_testsuite_property`` session-scoped fixture:

The ``record_testsuite_property`` session-scoped fixture can be used to add properties relevant
to all tests.

.. code-block:: python

    import pytest


    @pytest.fixture(scope="session", autouse=True)
    def log_global_env_facts(record_testsuite_property):
        record_testsuite_property("ARCH", "PPC")
        record_testsuite_property("STORAGE_TYPE", "CEPH")


    class TestMe:
        def test_foo(self):
            assert True

The fixture is a callable which receives ``name`` and ``value`` of a ``<property>`` tag
added at the test-suite level of the generated xml:

.. code-block:: xml

    <testsuite errors="0" failures="0" name="pytest" skipped="0" tests="1" time="0.006">
      <properties>
        <property name="ARCH" value="PPC"/>
        <property name="STORAGE_TYPE" value="CEPH"/>
      </properties>
      <testcase classname="test_me.TestMe" file="test_me.py" line="16" name="test_foo" time="0.000243663787842"/>
    </testsuite>

``name`` must be a string, ``value`` will be converted to a string and properly xml-escaped.

The generated XML is compatible with the latest ``xunit`` standard, contrary to `record_property`_
and `record_xml_attribute`_.


Sending test report to an online pastebin service
--------------------------------------------------

**Creating a URL for each test failure**:

.. code-block:: bash

    pytest --pastebin=failed

This will submit test run information to a remote Paste service and
provide a URL for each failure.  You may select tests as usual or add
for example ``-x`` if you only want to send one particular failure.

**Creating a URL for a whole test session log**:

.. code-block:: bash

    pytest --pastebin=all

Currently only pasting to the https://bpaste.net/ service is implemented.

.. versionchanged:: 5.2

If creating the URL fails for any reason, a warning is generated instead of failing the
entire test suite.

.. _jenkins: https://jenkins-ci.org
