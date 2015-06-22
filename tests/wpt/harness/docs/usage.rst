Getting Started
===============

Installing wptrunner
--------------------

The easiest way to install wptrunner is into a virtualenv, using pip::

  virtualenv wptrunner
  cd wptrunner
  source bin/activate
  pip install wptrunner

This will install the base dependencies for wptrunner, but not any
extra dependencies required to test against specific browsers. In
order to do this you must use use the extra requirements files in
``$VIRTUAL_ENV/requirements/requirements_browser.txt``. For example,
in order to test against Firefox you would have to run::

  pip install -r requirements/requirements_firefox.txt

If you intend to work on the code, the ``-e`` option to pip should be
used in combination with a source checkout i.e. inside a virtual
environment created as above::

  git clone https://github.com/w3c/wptrunner.git
  cd wptrunner
  pip install -e ./

In addition to the dependencies installed by pip, wptrunner requires
a copy of the web-platform-tests repository. That can be located
anywhere on the filesystem, but the easiest option is to put it within
the wptrunner checkout directory, as a subdirectory named ``tests``::

  git clone https://github.com/w3c/web-platform-tests.git tests

It is also necessary to generate a web-platform-tests ``MANIFEST.json``
file. It's recommended to put that within the wptrunner
checkout directory, in a subdirectory named ``meta``::

  mkdir meta
  cd tests
  python tools/scripts/manifest.py ../meta/MANIFEST.json

The ``MANIFEST.json`` file needs to be regenerated each time the
web-platform-tests checkout is updated. To aid with the update process
there is a tool called ``wptupdate``, which is described in
:ref:`wptupdate-label`.

Running the Tests
-----------------

A test run is started using the ``wptrunner`` command.  The command
takes multiple options, of which the following are most significant:

``--product`` (defaults to `firefox`)
  The product to test against: `b2g`, `chrome`, `firefox`, or `servo`.

``--binary`` (required if product is `firefox` or `servo`)
  The path to a binary file for the product (browser) to test against.

``--webdriver-binary`` (required if product is `chrome`)
  The path to a `*driver` binary; e.g., a `chromedriver` binary.

``--certutil-binary`` (required if product is `firefox` [#]_)
  The path to a `certutil` binary (for tests that must be run over https).

``--metadata`` (required only when not `using default paths`_)
  The path to a directory containing test metadata. [#]_

``--tests`` (required only when not `using default paths`_)
  The path to a directory containing a web-platform-tests checkout.

``--prefs-root`` (required only when testing a Firefox binary)
  The path to a directory containing Firefox test-harness preferences. [#]_

.. [#] The ``--certutil-binary`` option is required when the product is
   ``firefox`` unless ``--ssl-type=none`` is specified.

.. [#] The ``--metadata`` path is to a directory that contains:

  * a ``MANIFEST.json`` file (the web-platform-tests documentation has
    instructions on generating this file)
  * (optionally) any expectation files (see :ref:`wptupdate-label`)

.. [#] Example ``--prefs-root`` value: ``~/mozilla-central/testing/profiles``.

There are also a variety of other command-line options available; use
``--help`` to list them.

The following examples show how to start wptrunner with various options.

------------------
Starting wptrunner
------------------

To test a Firefox Nightly build in an OS X environment, you might start
wptrunner using something similar to the following example::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=~/mozilla-central/obj-x86_64-apple-darwin14.3.0/dist/Nightly.app/Contents/MacOS/firefox \
    --certutil-binary=~/mozilla-central/obj-x86_64-apple-darwin14.3.0/security/nss/cmd/certutil/certutil \
    --prefs-root=~/mozilla-central/testing/profiles


And to test a Chromium build in an OS X environment, you might start
wptrunner using something similar to the following example::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=~/chromium/src/out/Release/Chromium.app/Contents/MacOS/Chromium \
    --webdriver-binary=/usr/local/bin/chromedriver --product=chrome

--------------------
Running test subsets
--------------------

To restrict a test run just to tests in a particular web-platform-tests
subdirectory, specify the directory name in the positional arguments after
the options; for example, run just the tests in the `dom` subdirectory::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=/path/to/firefox --certutil-binary=/path/to/certutil \
    --prefs-root=/path/to/testing/profiles \
    dom

-------------------
Running in parallel
-------------------

To speed up the testing process, use the ``--processes`` option to have
wptrunner run multiple browser instances in parallel. For example, to
have wptrunner attempt to run tests against with six browser instances
in parallel, specify ``--processes=6``. But note that behaviour in this
mode is necessarily less deterministic than with ``--processes=1`` (the
default), so there may be more noise in the test results.

-------------------
Using default paths
-------------------

The (otherwise-required) ``--tests`` and ``--metadata`` command-line
options/flags be omitted if any configuration file is found that
contains a section specifying the ``tests`` and ``metadata`` keys.

See the `Configuration File`_ section for more information about
configuration files, including information about their expected
locations.

The content of the ``wptrunner.default.ini`` default configuration file
makes wptrunner look for tests (that is, a web-platform-tests checkout)
as a subdirectory of the current directory named ``tests``, and for
metadata files in a subdirectory of the current directory named ``meta``.

Output
------

wptrunner uses the :py:mod:`mozlog.structured` package for output. This
structures events such as test results or log messages as JSON objects
that can then be fed to other tools for interpretation. More details
about the message format are given in the
:py:mod:`mozlog.structured` documentation.

By default the raw JSON messages are dumped to stdout. This is
convenient for piping into other tools, but not ideal for humans
reading the output. :py:mod:`mozlog` comes with several other
formatters, which are accessible through command line options. The
general format of these options is ``--log-name=dest``, where ``name``
is the name of the format and ``dest`` is a path to a destination
file, or ``-`` for stdout. The raw JSON data is written by the ``raw``
formatter so, the default setup corresponds to ``--log-raw=-``.

A reasonable output format for humans is provided as ``mach``. So in
order to output the full raw log to a file and a human-readable
summary to stdout, one might pass the options::

  --log-raw=output.log --log-mach=-

Configuration File
------------------

wptrunner uses a ``.ini`` file to control some configuration
sections. The file has three sections; ``[products]``,
``[paths]`` and ``[web-platform-tests]``.

``[products]`` is used to
define the set of available products. By default this section is empty
which means that all the products distributed with wptrunner are
enabled (although their dependencies may not be installed). The set
of enabled products can be set by using the product name as the
key. For built in products the value is empty. It is also possible to
provide the path to a script implementing the browser functionality
e.g.::

  [products]
  chrome =
  netscape4 = path/to/netscape.py

``[paths]`` specifies the default paths for the tests and metadata,
relative to the config file. For example::

  [paths]
  tests = checkouts/web-platform-tests
  metadata = /home/example/wpt/metadata


``[web-platform-tests]`` is used to set the properties of the upstream
repository when updating the paths. ``remote_url`` specifies the git
url to pull from; ``branch`` the branch to sync against and
``sync_path`` the local path, relative to the configuration file, to
use when checking out the tests e.g.::

  [web-platform-tests]
  remote_url = https://github.com/w3c/web-platform-tests.git
  branch = master
  sync_path = sync

A configuration file must contain all the above fields; falling back
to the default values for unspecified fields is not yet supported.

The ``wptrunner`` and ``wptupdate`` commands will use configuration
files in the following order:

 * Any path supplied with a ``--config`` flag to the command.

 * A file called ``wptrunner.ini`` in the current directory

 * The default configuration file (``wptrunner.default.ini`` in the
   source directory)
