
Talks and Tutorials
==========================

.. sidebar:: Next Open Trainings

   `professional testing with pytest and tox <http://www.python-academy.com/courses/specialtopics/python_course_testing.html>`_, 27-29th June 2016, Freiburg, Germany

.. _`funcargs`: funcargs.html

Talks and blog postings
---------------------------------------------

.. _`tutorial1 repository`: http://bitbucket.org/pytest-dev/pytest-tutorial1/
.. _`pycon 2010 tutorial PDF`: http://bitbucket.org/pytest-dev/pytest-tutorial1/raw/tip/pytest-basic.pdf

- `pytest - Rapid Simple Testing, Florian Bruhin, Swiss Python Summit 2016
  <https://www.youtube.com/watch?v=rCBHkQ_LVIs>`_.

- `Improve your testing with Pytest and Mock, Gabe Hollombe, PyCon SG 2015
  <https://www.youtube.com/watch?v=RcN26hznmk4>`_.

- `Introduction to pytest, Andreas Pelme, EuroPython 2014
  <https://www.youtube.com/watch?v=LdVJj65ikRY>`_.

- `Advanced Uses of py.test Fixtures, Floris Bruynooghe, EuroPython
  2014 <https://www.youtube.com/watch?v=IBC_dxr-4ps>`_.

- `Why i use py.test and maybe you should too, Andy Todd, Pycon AU 2013
  <https://www.youtube.com/watch?v=P-AhpukDIik>`_

- `3-part blog series about pytest from @pydanny alias Daniel Greenfeld (January
  2014) <http://pydanny.com/pytest-no-boilerplate-testing.html>`_

- `pytest: helps you write better Django apps, Andreas Pelme, DjangoCon
  Europe 2014 <https://www.youtube.com/watch?v=aaArYVh6XSM>`_.

- :ref:`fixtures`

- `Testing Django Applications with pytest, Andreas Pelme, EuroPython
  2013 <https://www.youtube.com/watch?v=aUf8Fkb7TaY>`_.

- `Testes pythonics com py.test, Vinicius Belchior Assef Neto, Plone
  Conf 2013, Brazil <https://www.youtube.com/watch?v=QUKoq2K7bis>`_.

- `Introduction to py.test fixtures, FOSDEM 2013, Floris Bruynooghe
  <https://www.youtube.com/watch?v=bJhRW4eZMco>`_.

- `pytest feature and release highlights, Holger Krekel (GERMAN, October 2013)
  <http://pyvideo.org/video/2429/pytest-feature-and-new-release-highlights>`_

- `pytest introduction from Brian Okken (January 2013)
  <http://pythontesting.net/framework/pytest-introduction/>`_

- `monkey patching done right`_ (blog post, consult `monkeypatch
  plugin`_ for up-to-date API)

Test parametrization:

- `generating parametrized tests with funcargs`_ (uses deprecated ``addcall()`` API.
- `test generators and cached setup`_
- `parametrizing tests, generalized`_ (blog post)
- `putting test-hooks into local or global plugins`_ (blog post)

Assertion introspection:

- `(07/2011) Behind the scenes of pytest's new assertion rewriting
  <http://pybites.blogspot.com/2011/07/behind-scenes-of-pytests-new-assertion.html>`_

Distributed testing:

- `simultaneously test your code on all platforms`_ (blog entry)

Plugin specific examples:

- `skipping slow tests by default in pytest`_ (blog entry)

- `many examples in the docs for plugins`_

.. _`skipping slow tests by default in pytest`: http://bruynooghe.blogspot.com/2009/12/skipping-slow-test-by-default-in-pytest.html
.. _`many examples in the docs for plugins`: plugin/index.html
.. _`monkeypatch plugin`: plugin/monkeypatch.html
.. _`application setup in test functions with funcargs`: funcargs.html#appsetup
.. _`simultaneously test your code on all platforms`: http://tetamap.wordpress.com/2009/03/23/new-simultanously-test-your-code-on-all-platforms/
.. _`monkey patching done right`: http://tetamap.wordpress.com/2009/03/03/monkeypatching-in-unit-tests-done-right/
.. _`putting test-hooks into local or global plugins`: http://tetamap.wordpress.com/2009/05/14/putting-test-hooks-into-local-and-global-plugins/
.. _`parametrizing tests, generalized`: http://tetamap.wordpress.com/2009/05/13/parametrizing-python-tests-generalized/
.. _`generating parametrized tests with funcargs`: funcargs.html#test-generators
.. _`test generators and cached setup`: http://bruynooghe.blogspot.com/2010/06/pytest-test-generators-and-cached-setup.html

Older conference talks and tutorials
----------------------------------------

- `pycon australia 2012 pytest talk from Brianna Laugher
  <http://2012.pycon-au.org/schedule/52/view_talk?day=sunday>`_ (`video <http://www.youtube.com/watch?v=DTNejE9EraI>`_, `slides <http://www.slideshare.net/pfctdayelise/funcargs-other-fun-with-pytest>`_, `code <https://gist.github.com/3386951>`_)
- `pycon 2012 US talk video from Holger Krekel <http://www.youtube.com/watch?v=9LVqBQcFmyw>`_
- `pycon 2010 tutorial PDF`_ and `tutorial1 repository`_

- `ep2009-rapidtesting.pdf`_ tutorial slides (July 2009):

  - testing terminology
  - basic pytest usage, file system layout
  - test function arguments (funcargs_) and test fixtures
  - existing plugins
  - distributed testing

- `ep2009-pytest.pdf`_ 60 minute pytest talk, highlighting unique features and a roadmap (July 2009)

- `pycon2009-pytest-introduction.zip`_ slides and files, extended version of pytest basic introduction, discusses more options, also introduces old-style xUnit setup, looponfailing and other features.

- `pycon2009-pytest-advanced.pdf`_ contain a slightly older version of funcargs and distributed testing, compared to the EuroPython 2009 slides.

.. _`ep2009-rapidtesting.pdf`: http://codespeak.net/download/py/ep2009-rapidtesting.pdf
.. _`ep2009-pytest.pdf`: http://codespeak.net/download/py/ep2009-pytest.pdf
.. _`pycon2009-pytest-introduction.zip`: http://codespeak.net/download/py/pycon2009-pytest-introduction.zip
.. _`pycon2009-pytest-advanced.pdf`: http://codespeak.net/download/py/pycon2009-pytest-advanced.pdf
