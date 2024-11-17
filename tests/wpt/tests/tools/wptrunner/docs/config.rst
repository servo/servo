wptrunner Configuration
=======================

wptrunner can be configured using two mechanisms:

 * Command line arguments

 * A ``wptrunner.ini`` configuration file

Command Line Arguments
----------------------

Command line arguments are the most common way of configuring
wptrunner. The current list of command line arguments can be seen by
starting wptrunner with the ``--help`` command line argument.

Command line arguments override options given in the configuration file.


Configuration File
------------------

A configuration file can be passed using the ``--config`` command line
argument. If no argument is supplied then ``wptrunner.ini`` in the
current working directory will be used, if it exists, otherwise
``wptrunner.default.ini`` in the wptrunner directory. Only a single
configuration file is used.

Typicaly frontends to wptrunner are expected to pass in their own
configuration file.

The configuration file contains the following known paths and sections:

:paths:
    Data about default paths to use.

    :prefs:
        Path to profile root directory. Equivalent to the
        ``--profile-root`` command line argument.

    :run_info:
        Path to the directory containing extra run info JSON
        files to add to the run info data. Equivalent to the ``--run-info``
        command line argument.

    :ws_extra:
        Semicolon-separated list of extra paths to use for
        websockets handlers. Equivalent to the ``--ws-extra`` command line
        argument.

:web-platform-tests:
    Data about the web-platform-tests repository. This is only used by the
    repository sync code and can be considered deprecated.

    :remote_url: URL of the wpt repository to sync from
    :branch: Branch name to sync from
    :sync_path: Directory to use when performing a sync

In addition the command line allows specifying *multiple* sections
each corresponding to a test manifest. These are named
``manifest:[name]``. The ``name`` is arbitary, but must be unique in
the file. At least one such section is required so that wptrunner
knows where to find some tests.

:manifest\:[name]:
    Data about tests in a given subtree.

    :tests: Path to the root of the subtree containing tests.
    :meta: Path to the corresponding metadata directory.
    :url_base: URL prefix to for the tests in this manifest. This
               should be ``/`` for the default manifest but must be
               different for other manifests.

For example a vendor with both upstream web-platform-tests under an
``upstream`` subtree, and vendor-specific web-platform-tests under a
``local`` substree, might have a configuration like::

  [manifest:upstream]
  tests = upstream/tests
  metadata = upstream/meta
  url_base = /

  [manifest:vendor]
  tests = local/tests
  metadata = local/meta
  url_base = /_local/
