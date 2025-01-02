.. highlight:: python
.. _`goodpractices`:

Good Integration Practices
=================================================

Install package with pip
-------------------------------------------------

For development, we recommend you use :mod:`venv` for virtual environments and
:doc:`pip:index` for installing your application and any dependencies,
as well as the ``pytest`` package itself.
This ensures your code and dependencies are isolated from your system Python installation.

Create a ``pyproject.toml`` file in the root of your repository as described in
:doc:`packaging:tutorials/packaging-projects`.
The first few lines should look like this:

.. code-block:: toml

    [build-system]
    requires = ["hatchling"]
    build-backend = "hatchling.build"

    [project]
    name = "PACKAGENAME"
    version = "PACKAGEVERSION"

where ``PACKAGENAME`` and ``PACKAGEVERSION`` are the name and version of your package respectively.

You can then install your package in "editable" mode by running from the same directory:

.. code-block:: bash

    pip install -e .

which lets you change your source code (both tests and application) and rerun tests at will.

.. _`test discovery`:
.. _`Python test discovery`:

Conventions for Python test discovery
-------------------------------------------------

``pytest`` implements the following standard test discovery:

* If no arguments are specified then collection starts from :confval:`testpaths`
  (if configured) or the current directory. Alternatively, command line arguments
  can be used in any combination of directories, file names or node ids.
* Recurse into directories, unless they match :confval:`norecursedirs`.
* In those directories, search for ``test_*.py`` or ``*_test.py`` files, imported by their `test package name`_.
* From those files, collect test items:

  * ``test`` prefixed test functions or methods outside of class.
  * ``test`` prefixed test functions or methods inside ``Test`` prefixed test classes (without an ``__init__`` method). Methods decorated with ``@staticmethod`` and ``@classmethods`` are also considered.

For examples of how to customize your test discovery :doc:`/example/pythoncollection`.

Within Python modules, ``pytest`` also discovers tests using the standard
:ref:`unittest.TestCase <unittest.TestCase>` subclassing technique.


.. _`test layout`:

Choosing a test layout
----------------------

``pytest`` supports two common test layouts:

Tests outside application code
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Putting tests into an extra directory outside your actual application code
might be useful if you have many functional tests or for other reasons want
to keep tests separate from actual application code (often a good idea):

.. code-block:: text

    pyproject.toml
    src/
        mypkg/
            __init__.py
            app.py
            view.py
    tests/
        test_app.py
        test_view.py
        ...

This has the following benefits:

* Your tests can run against an installed version after executing ``pip install .``.
* Your tests can run against the local copy with an editable install after executing ``pip install --editable .``.

For new projects, we recommend to use ``importlib`` :ref:`import mode <import-modes>`
(see which-import-mode_ for a detailed explanation).
To this end, add the following to your ``pyproject.toml``:

.. code-block:: toml

    [tool.pytest.ini_options]
    addopts = [
        "--import-mode=importlib",
    ]

.. _src-layout:

Generally, but especially if you use the default import mode ``prepend``,
it is **strongly** suggested to use a ``src`` layout.
Here, your application root package resides in a sub-directory of your root,
i.e. ``src/mypkg/`` instead of ``mypkg``.

This layout prevents a lot of common pitfalls and has many benefits,
which are better explained in this excellent `blog post`_ by Ionel Cristian Mărieș.

.. _blog post: https://blog.ionelmc.ro/2014/05/25/python-packaging/#the-structure>

.. note::

    If you do not use an editable install and use the ``src`` layout as above you need to extend the Python's
    search path for module files to execute the tests against the local copy directly. You can do it in an
    ad-hoc manner by setting the ``PYTHONPATH`` environment variable:

    .. code-block:: bash

       PYTHONPATH=src pytest

    or in a permanent manner by using the :confval:`pythonpath` configuration variable and adding the
    following to your ``pyproject.toml``:

    .. code-block:: toml

        [tool.pytest.ini_options]
        pythonpath = "src"

.. note::

    If you do not use an editable install and not use the ``src`` layout (``mypkg`` directly in the root
    directory) you can rely on the fact that Python by default puts the current directory in ``sys.path`` to
    import your package and run ``python -m pytest`` to execute the tests against the local copy directly.

    See :ref:`pytest vs python -m pytest` for more information about the difference between calling ``pytest`` and
    ``python -m pytest``.

Tests as part of application code
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Inlining test directories into your application package
is useful if you have direct relation between tests and application modules and
want to distribute them along with your application:

.. code-block:: text

    pyproject.toml
    [src/]mypkg/
        __init__.py
        app.py
        view.py
        tests/
            __init__.py
            test_app.py
            test_view.py
            ...

In this scheme, it is easy to run your tests using the ``--pyargs`` option:

.. code-block:: bash

    pytest --pyargs mypkg

``pytest`` will discover where ``mypkg`` is installed and collect tests from there.

Note that this layout also works in conjunction with the ``src`` layout mentioned in the previous section.


.. note::

    You can use namespace packages (PEP420) for your application
    but pytest will still perform `test package name`_ discovery based on the
    presence of ``__init__.py`` files.  If you use one of the
    two recommended file system layouts above but leave away the ``__init__.py``
    files from your directories, it should just work.  From
    "inlined tests", however, you will need to use absolute imports for
    getting at your application code.

.. _`test package name`:

.. note::

    In ``prepend`` and ``append`` import-modes, if pytest finds a ``"a/b/test_module.py"``
    test file while recursing into the filesystem it determines the import name
    as follows:

    * determine ``basedir``: this is the first "upward" (towards the root)
      directory not containing an ``__init__.py``.  If e.g. both ``a``
      and ``b`` contain an ``__init__.py`` file then the parent directory
      of ``a`` will become the ``basedir``.

    * perform ``sys.path.insert(0, basedir)`` to make the test module
      importable under the fully qualified import name.

    * ``import a.b.test_module`` where the path is determined
      by converting path separators ``/`` into "." characters.  This means
      you must follow the convention of having directory and file
      names map directly to the import names.

    The reason for this somewhat evolved importing technique is
    that in larger projects multiple test modules might import
    from each other and thus deriving a canonical import name helps
    to avoid surprises such as a test module getting imported twice.

    With ``--import-mode=importlib`` things are less convoluted because
    pytest doesn't need to change ``sys.path`` or ``sys.modules``, making things
    much less surprising.


.. _which-import-mode:

Choosing an import mode
^^^^^^^^^^^^^^^^^^^^^^^

For historical reasons, pytest defaults to the ``prepend`` :ref:`import mode <import-modes>`
instead of the ``importlib`` import mode we recommend for new projects.
The reason lies in the way the ``prepend`` mode works:

Since there are no packages to derive a full package name from,
``pytest`` will import your test files as *top-level* modules.
The test files in the first example (:ref:`src layout <src-layout>`) would be imported as
``test_app`` and ``test_view`` top-level modules by adding ``tests/`` to ``sys.path``.

This results in a drawback compared to the import mode ``importlib``:
your test files must have **unique names**.

If you need to have test modules with the same name,
as a workaround you might add ``__init__.py`` files to your ``tests`` folder and subfolders,
changing them to packages:

.. code-block:: text

    pyproject.toml
    mypkg/
        ...
    tests/
        __init__.py
        foo/
            __init__.py
            test_view.py
        bar/
            __init__.py
            test_view.py

Now pytest will load the modules as ``tests.foo.test_view`` and ``tests.bar.test_view``,
allowing you to have modules with the same name.
But now this introduces a subtle problem:
in order to load the test modules from the ``tests`` directory,
pytest prepends the root of the repository to ``sys.path``,
which adds the side-effect that now ``mypkg`` is also importable.

This is problematic if you are using a tool like tox_ to test your package in a virtual environment,
because you want to test the *installed* version of your package,
not the local code from the repository.

The ``importlib`` import mode does not have any of the drawbacks above,
because ``sys.path`` is not changed when importing test modules.


.. _`buildout`: http://www.buildout.org/en/latest/

.. _`use tox`:

tox
---

Once you are done with your work and want to make sure that your actual
package passes all tests you may want to look into :doc:`tox <tox:index>`, the
virtualenv test automation tool.
``tox`` helps you to setup virtualenv environments with pre-defined
dependencies and then executing a pre-configured test command with
options.  It will run tests against the installed package and not
against your source code checkout, helping to detect packaging
glitches.

Do not run via setuptools
-------------------------

Integration with setuptools is **not recommended**,
i.e. you should not be using ``python setup.py test`` or ``pytest-runner``,
and may stop working in the future.

This is deprecated since it depends on deprecated features of setuptools
and relies on features that break security mechanisms in pip.
For example 'setup_requires' and 'tests_require' bypass ``pip --require-hashes``.
For more information and migration instructions,
see the `pytest-runner notice <https://github.com/pytest-dev/pytest-runner#deprecation-notice>`_.
See also `pypa/setuptools#1684 <https://github.com/pypa/setuptools/issues/1684>`_.

setuptools intends to
`remove the test command <https://github.com/pypa/setuptools/issues/931>`_.

Checking with flake8-pytest-style
---------------------------------

In order to ensure that pytest is being used correctly in your project,
it can be helpful to use the `flake8-pytest-style <https://github.com/m-burst/flake8-pytest-style>`_ flake8 plugin.

flake8-pytest-style checks for common mistakes and coding style violations in pytest code,
such as incorrect use of fixtures, test function names, and markers.
By using this plugin, you can catch these errors early in the development process
and ensure that your pytest code is consistent and easy to maintain.

A list of the lints detected by flake8-pytest-style can be found on its `PyPI page <https://pypi.org/project/flake8-pytest-style/>`_.

.. note::

    flake8-pytest-style is not an official pytest project. Some of the rules enforce certain style choices, such as using `@pytest.fixture()` over `@pytest.fixture`, but you can configure the plugin to fit your preferred style.
