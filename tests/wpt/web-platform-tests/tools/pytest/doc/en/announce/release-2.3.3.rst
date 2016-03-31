pytest-2.3.3: integration fixes, py24 suport, ``*/**`` shown in traceback
===========================================================================

pytest-2.3.3 is a another stabilization release of the py.test tool
which offers uebersimple assertions, scalable fixture mechanisms
and deep customization for testing with Python.  Particularly,
this release provides:

- integration fixes and improvements related to flask, numpy, nose, 
  unittest, mock

- makes pytest work on py24 again (yes, people sometimes still need to use it)

- show ``*,**`` args in pytest tracebacks

Thanks to Manuel Jacob, Thomas Waldmann, Ronny Pfannschmidt, Pavel Repin
and Andreas Taumoefolau for providing patches and all for the issues.

See 

     http://pytest.org/

for general information.  To install or upgrade pytest:

    pip install -U pytest # or
    easy_install -U pytest

best,
holger krekel

Changes between 2.3.2 and 2.3.3
-----------------------------------

- fix issue214 - parse modules that contain special objects like e. g.
  flask's request object which blows up on getattr access if no request
  is active. thanks Thomas Waldmann.

- fix issue213 - allow to parametrize with values like numpy arrays that
  do not support an __eq__ operator

- fix issue215 - split test_python.org into multiple files

- fix issue148 - @unittest.skip on classes is now recognized and avoids
  calling setUpClass/tearDownClass, thanks Pavel Repin

- fix issue209 - reintroduce python2.4 support by depending on newer
  pylib which re-introduced statement-finding for pre-AST interpreters

- nose support: only call setup if its a callable, thanks Andrew
  Taumoefolau

- fix issue219 - add py2.4-3.3 classifiers to TROVE list

- in tracebacks *,** arg values are now shown next to normal arguments
  (thanks Manuel Jacob)

- fix issue217 - support mock.patch with pytest's fixtures - note that
  you need either mock-1.0.1 or the python3.3 builtin unittest.mock.

- fix issue127 - improve documentation for pytest_addoption() and
  add a ``config.getoption(name)`` helper function for consistency.

