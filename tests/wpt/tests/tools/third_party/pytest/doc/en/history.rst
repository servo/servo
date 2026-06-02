History
=======

pytest has a long and interesting history. The `first commit
<https://github.com/pytest-dev/pytest/commit/5992a8ef21424d7571305a8d7e2a3431ee7e1e23>`__
in this repository is from January 2007, and even that commit alone already
tells a lot: The repository originally was from the :pypi:`py`
library (later split off to pytest), and it
originally was a SVN revision, migrated to Mercurial, and finally migrated to
git.

However, the commit says “create the new development trunk” and is
already quite big: *435 files changed, 58640 insertions(+)*. This is because
pytest originally was born as part of `PyPy <https://www.pypy.org/>`__, to make
it easier to write tests for it. Here's how it evolved from there to its own
project:


-  Late 2002 / early 2003, `PyPy was
   born <https://morepypy.blogspot.com/2018/09/the-first-15-years-of-pypy.html>`__.
-  Like that blog post mentioned, from very early on, there was a big
   focus on testing. There were various ``testsupport`` files on top of
   unittest.py, and as early as June 2003, Holger Krekel (:user:`hpk42`)
   `refactored <https://mail.python.org/pipermail/pypy-dev/2003-June/000787.html>`__
   its test framework to clean things up (``pypy.tool.test``, but still
   on top of ``unittest.py``, with nothing pytest-like yet).
-  In December 2003, there was `another
   iteration <https://foss.heptapod.net/pypy/pypy/-/commit/02752373e1b29d89c6bb0a97e5f940caa22bdd63>`__
   at improving their testing situation, by Stefan Schwarzer, called
   ``pypy.tool.newtest``.
-  However, it didn’t seem to be around for long, as around June/July
   2004, efforts started on a thing called ``utest``, offering plain
   assertions. This seems like the start of something pytest-like, but
   unfortunately, it's unclear where the test runner's code was at the time.
   The closest thing still around is `this
   file <https://foss.heptapod.net/pypy/pypy/-/commit/0735f9ed287ec20950a7dd0a16fc10810d4f6847>`__,
   but that doesn’t seem like a complete test runner at all. What can be seen
   is that there were `various
   efforts <https://foss.heptapod.net/pypy/pypy/-/commits/branch/default?utf8=%E2%9C%93&search=utest>`__
   by Laura Creighton and Samuele Pedroni (:user:`pedronis`) at automatically
   converting existing tests to the new ``utest`` framework.
-  Around the same time, for Europython 2004, @hpk42 `started a
   project <http://web.archive.org/web/20041020215353/http://codespeak.net/svn/user/hpk/talks/std-talk.txt>`__
   originally called “std”, intended to be a “complementary standard
   library” - already laying out the principles behind what later became
   pytest:

       -  current “batteries included” are very useful, but

          -  some of them are written in a pretty much java-like style,
             especially the unittest-framework
          -  […]
          -  the best API is one that doesn’t exist

       […]

       -  a testing package should require as few boilerplate code as
          possible and offer much flexibility
       -  it should provide premium quality tracebacks and debugging aid

       […]

       -  first of all … forget about limited “assertXYZ APIs” and use the
          real thing, e.g.::

              assert x == y

       -  this works with plain python but you get unhelpful “assertion
          failed” errors with no information

       -  std.utest (magic!) actually reinterprets the assertion expression
          and offers detailed information about underlying values

-  In September 2004, the ``py-dev`` mailinglist gets born, which `is
   now <https://mail.python.org/pipermail/pytest-dev/>`__ ``pytest-dev``,
   but thankfully with all the original archives still intact.

-  Around September/October 2004, the ``std`` project `was renamed
   <https://mail.python.org/pipermail/pypy-dev/2004-September/001565.html>`__ to
   ``py`` and ``std.utest`` became ``py.test``. This is also the first time the
   `entire source
   code <https://foss.heptapod.net/pypy/pypy/-/commit/42cf50c412026028e20acd23d518bd92e623ac11>`__,
   seems to be available, with much of the API still being around today:

   -  ``py.path.local``, which is being phased out of pytest (in favour of
      pathlib) some 16-17 years later
   -  The idea of the collection tree, including ``Collector``,
      ``FSCollector``, ``Directory``, ``PyCollector``, ``Module``,
      ``Class``
   -  Arguments like ``-x`` / ``--exitfirst``, ``-l`` /
      ``--showlocals``, ``--fulltrace``, ``--pdb``, ``-S`` /
      ``--nocapture`` (``-s`` / ``--capture=off`` today),
      ``--collectonly`` (``--collect-only`` today)

-  In the same month, the ``py`` library `gets split off
   <https://foss.heptapod.net/pypy/pypy/-/commit/6bdafe9203ad92eb259270b267189141c53bce33>`__
   from ``PyPy``

-  It seemed to get rather quiet for a while, and little seemed to happen
   between October 2004 (removing ``py`` from PyPy) and January
   2007 (first commit in the now-pytest repository). However, there were
   various discussions about features/ideas on the mailinglist, and
   :pypi:`a couple of releases <py/0.8.0-alpha2/#history>` every
   couple of months:

   -  March 2006: py 0.8.0-alpha2
   -  May 2007: py 0.9.0
   -  March 2008: py 0.9.1 (first release to be found `in the pytest
      changelog <https://github.com/pytest-dev/pytest/blob/main/doc/en/changelog.rst#091>`__!)
   -  August 2008: py 0.9.2

-  In August 2009, py 1.0.0 was released, `introducing a lot of
   fundamental
   features <https://holgerkrekel.net/2009/08/04/pylib-1-0-0-released-the-testing-with-python-innovations-continue/>`__:

   -  funcargs/fixtures
   -  A `plugin
      architecture <http://web.archive.org/web/20090629032718/https://codespeak.net/py/dist/test/extend.html>`__
      which still looks very much the same today!
   -  Various `default
      plugins <http://web.archive.org/web/20091005181132/https://codespeak.net/py/dist/test/plugin/index.html>`__,
      including
      `monkeypatch <http://web.archive.org/web/20091012022829/http://codespeak.net/py/dist/test/plugin/how-to/monkeypatch.html>`__

-  Even back there, the
   `FAQ <http://web.archive.org/web/20091005222413/http://codespeak.net/py/dist/faq.html>`__
   said:

       Clearly, [a second standard library] was ambitious and the naming has
       maybe haunted the project rather than helping it. There may be a
       project name change and possibly a split up into different projects
       sometime.

   and that finally happened in November 2010, when pytest 2.0.0 `was
   released <https://mail.python.org/pipermail/pytest-dev/2010-November/001687.html>`__
   as a package separate from ``py`` (but still called ``py.test``).

-  In August 2016, pytest 3.0.0 :std:ref:`was released <release-3.0.0>`,
   which adds ``pytest`` (rather than ``py.test``) as the recommended
   command-line entry point

Due to this history, it's difficult to answer the question when pytest was started.
It depends what point should really be seen as the start of it all. One
possible interpretation is to  pick Europython 2004, i.e. around June/July
2004.
