pytest-2.6.2: few fixes and cx_freeze support
===========================================================================

pytest is a mature Python testing tool with more than a 1100 tests
against itself, passing on many different interpreters and platforms.
This release is drop-in compatible to 2.5.2 and 2.6.X.  It also
brings support for including pytest with cx_freeze or similar
freezing tools into your single-file app distribution.  For details
see the CHANGELOG below.

See docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed, among them:

    Floris Bruynooghe
    Benjamin Peterson
    Bruno Oliveira

have fun,
holger krekel

2.6.2
-----------

- Added function pytest.freeze_includes(), which makes it easy to embed
  pytest into executables using tools like cx_freeze.
  See docs for examples and rationale. Thanks Bruno Oliveira.

- Improve assertion rewriting cache invalidation precision.

- fixed issue561: adapt autouse fixture example for python3.

- fixed issue453: assertion rewriting issue with __repr__ containing
  "\n{", "\n}" and "\n~".

- fix issue560: correctly display code if an "else:" or "finally:" is
  followed by statements on the same line.

- Fix example in monkeypatch documentation, thanks t-8ch.

- fix issue572: correct tmpdir doc example for python3.

- Do not mark as universal wheel because Python 2.6 is different from
  other builds due to the extra argparse dependency.  Fixes issue566.
  Thanks sontek.

