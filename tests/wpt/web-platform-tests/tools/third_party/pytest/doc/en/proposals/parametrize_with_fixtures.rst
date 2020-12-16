:orphan:

===================================
PROPOSAL: Parametrize with fixtures
===================================

.. warning::

    This document outlines a proposal around using fixtures as input
    of parametrized tests or fixtures.

Problem
-------

As a user I have functional tests that I would like to run against various
scenarios.

In this particular example we want to generate a new project based on a
cookiecutter template. We want to test default values but also data that
emulates user input.

- use default values

- emulate user input

  - specify 'author'

  - specify 'project_slug'

  - specify 'author' and 'project_slug'

This is how a functional test could look like:

.. code-block:: python

    import pytest


    @pytest.fixture
    def default_context():
        return {"extra_context": {}}


    @pytest.fixture(
        params=[
            {"author": "alice"},
            {"project_slug": "helloworld"},
            {"author": "bob", "project_slug": "foobar"},
        ]
    )
    def extra_context(request):
        return {"extra_context": request.param}


    @pytest.fixture(params=["default", "extra"])
    def context(request):
        if request.param == "default":
            return request.getfuncargvalue("default_context")
        else:
            return request.getfuncargvalue("extra_context")


    def test_generate_project(cookies, context):
        """Call the cookiecutter API to generate a new project from a
        template.
        """
        result = cookies.bake(extra_context=context)

        assert result.exit_code == 0
        assert result.exception is None
        assert result.project.isdir()


Issues
------

* By using ``request.getfuncargvalue()`` we rely on actual fixture function
  execution to know what fixtures are involved, due to its dynamic nature
* More importantly, ``request.getfuncargvalue()`` cannot be combined with
  parametrized fixtures, such as ``extra_context``
* This is very inconvenient if you wish to extend an existing test suite by
  certain parameters for fixtures that are already used by tests

pytest version 3.0 reports an error if you try to run above code::

    Failed: The requested fixture has no parameter defined for the current
    test.

    Requested fixture 'extra_context'


Proposed solution
-----------------

A new function that can be used in modules can be used to dynamically define
fixtures from existing ones.

.. code-block:: python

    pytest.define_combined_fixture(
        name="context", fixtures=["default_context", "extra_context"]
    )

The new fixture ``context`` inherits the scope from the used fixtures and yield
the following values.

- ``{}``

- ``{'author': 'alice'}``

- ``{'project_slug': 'helloworld'}``

- ``{'author': 'bob', 'project_slug': 'foobar'}``

Alternative approach
--------------------

A new helper function named ``fixture_request`` would tell pytest to yield
all parameters marked as a fixture.

.. note::

    The `pytest-lazy-fixture <https://pypi.org/project/pytest-lazy-fixture/>`_ plugin implements a very
    similar solution to the proposal below, make sure to check it out.

.. code-block:: python

    @pytest.fixture(
        params=[
            pytest.fixture_request("default_context"),
            pytest.fixture_request("extra_context"),
        ]
    )
    def context(request):
        """Returns all values for ``default_context``, one-by-one before it
        does the same for ``extra_context``.

        request.param:
            - {}
            - {'author': 'alice'}
            - {'project_slug': 'helloworld'}
            - {'author': 'bob', 'project_slug': 'foobar'}
        """
        return request.param

The same helper can be used in combination with ``pytest.mark.parametrize``.

.. code-block:: python


    @pytest.mark.parametrize(
        "context, expected_response_code",
        [
            (pytest.fixture_request("default_context"), 0),
            (pytest.fixture_request("extra_context"), 0),
        ],
    )
    def test_generate_project(cookies, context, exit_code):
        """Call the cookiecutter API to generate a new project from a
        template.
        """
        result = cookies.bake(extra_context=context)

        assert result.exit_code == exit_code
