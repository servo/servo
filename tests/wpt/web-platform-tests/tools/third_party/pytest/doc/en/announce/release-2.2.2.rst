pytest-2.2.2: bug fixes
===========================================================================

pytest-2.2.2 (updated to 2.2.3 to fix packaging issues) is a minor
backward-compatible release of the versatile py.test testing tool.   It
contains bug fixes and a few refinements particularly to reporting with
"--collectonly", see below for betails.  

For general information see here:

     http://pytest.org/

To install or upgrade pytest:

    pip install -U pytest # or
    easy_install -U pytest

Special thanks for helping on this release to Ronny Pfannschmidt
and Ralf Schmitt and the contributors of issues.

best,
holger krekel


Changes between 2.2.1 and 2.2.2
----------------------------------------

- fix issue101: wrong args to unittest.TestCase test function now
  produce better output
- fix issue102: report more useful errors and hints for when a 
  test directory was renamed and some pyc/__pycache__ remain
- fix issue106: allow parametrize to be applied multiple times
  e.g. from module, class and at function level.
- fix issue107: actually perform session scope finalization
- don't check in parametrize if indirect parameters are funcarg names
- add chdir method to monkeypatch funcarg
- fix crash resulting from calling monkeypatch undo a second time
- fix issue115: make --collectonly robust against early failure
  (missing files/directories)
- "-qq --collectonly" now shows only files and the number of tests in them
- "-q --collectonly" now shows test ids 
- allow adding of attributes to test reports such that it also works
  with distributed testing (no upgrade of pytest-xdist needed)
