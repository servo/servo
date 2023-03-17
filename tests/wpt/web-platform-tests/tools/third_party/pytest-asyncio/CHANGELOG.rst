=========
Changelog
=========

0.19.0 (22-07-13)
=================
- BREAKING: The default ``asyncio_mode`` is now *strict*. `#293 <https://github.com/pytest-dev/pytest-asyncio/issues/293>`_
- Removes `setup.py` since all relevant configuration is present `setup.cfg`. Users requiring an editable installation of pytest-asyncio need to use pip v21.1 or newer. `#283 <https://github.com/pytest-dev/pytest-asyncio/issues/283>`_
- Declare support for Python 3.11.

0.18.3 (22-03-25)
=================
- Adds `pytest-trio <https://pypi.org/project/pytest-trio/>`_ to the test dependencies
- Fixes a bug that caused pytest-asyncio to try to set up async pytest_trio fixtures in strict mode. `#298 <https://github.com/pytest-dev/pytest-asyncio/issues/298>`_

0.18.2 (22-03-03)
=================
- Fix asyncio auto mode not marking static methods. `#295 <https://github.com/pytest-dev/pytest-asyncio/issues/295>`_
- Fix a compatibility issue with Hypothesis 6.39.0. `#302 <https://github.com/pytest-dev/pytest-asyncio/issues/302>`_

0.18.1 (22-02-10)
=================
- Fixes a regression that prevented async fixtures from working in synchronous tests. `#286 <https://github.com/pytest-dev/pytest-asyncio/issues/286>`_

0.18.0 (22-02-07)
=================

- Raise a warning if @pytest.mark.asyncio is applied to non-async function. `#275 <https://github.com/pytest-dev/pytest-asyncio/issues/275>`_
- Support parametrized ``event_loop`` fixture. `#278 <https://github.com/pytest-dev/pytest-asyncio/issues/278>`_

0.17.2 (22-01-17)
=================

- Require ``typing-extensions`` on Python<3.8 only. `#269 <https://github.com/pytest-dev/pytest-asyncio/issues/269>`_
- Fix a regression in tests collection introduced by 0.17.1, the plugin works fine with non-python tests again. `#267 <https://github.com/pytest-dev/pytest-asyncio/issues/267>`_


0.17.1 (22-01-16)
=================
- Fixes a bug that prevents async Hypothesis tests from working without explicit ``asyncio`` marker when ``--asyncio-mode=auto`` is set. `#258 <https://github.com/pytest-dev/pytest-asyncio/issues/258>`_
- Fixed a bug that closes the default event loop if the loop doesn't exist `#257 <https://github.com/pytest-dev/pytest-asyncio/issues/257>`_
- Added type annotations. `#198 <https://github.com/pytest-dev/pytest-asyncio/issues/198>`_
- Show asyncio mode in pytest report headers. `#266 <https://github.com/pytest-dev/pytest-asyncio/issues/266>`_
- Relax ``asyncio_mode`` type definition; it allows to support pytest 6.1+. `#262 <https://github.com/pytest-dev/pytest-asyncio/issues/262>`_

0.17.0 (22-01-13)
=================
- `pytest-asyncio` no longer alters existing event loop policies. `#168 <https://github.com/pytest-dev/pytest-asyncio/issues/168>`_, `#188 <https://github.com/pytest-dev/pytest-asyncio/issues/168>`_
- Drop support for Python 3.6
- Fixed an issue when pytest-asyncio was used in combination with `flaky` or inherited asynchronous Hypothesis tests. `#178 <https://github.com/pytest-dev/pytest-asyncio/issues/178>`_ `#231 <https://github.com/pytest-dev/pytest-asyncio/issues/231>`_
- Added `flaky <https://pypi.org/project/flaky/>`_ to test dependencies
- Added ``unused_udp_port`` and ``unused_udp_port_factory`` fixtures (similar to ``unused_tcp_port`` and ``unused_tcp_port_factory`` counterparts. `#99 <https://github.com/pytest-dev/pytest-asyncio/issues/99>`_
- Added the plugin modes: *strict*, *auto*, and *legacy*. See `documentation <https://github.com/pytest-dev/pytest-asyncio#modes>`_ for details. `#125 <https://github.com/pytest-dev/pytest-asyncio/issues/125>`_
- Correctly process ``KeyboardInterrupt`` during async fixture setup phase `#219 <https://github.com/pytest-dev/pytest-asyncio/issues/219>`_

0.16.0 (2021-10-16)
===================
- Add support for Python 3.10

0.15.1 (2021-04-22)
===================
- Hotfix for errors while closing event loops while replacing them.
  `#209 <https://github.com/pytest-dev/pytest-asyncio/issues/209>`_
  `#210 <https://github.com/pytest-dev/pytest-asyncio/issues/210>`_

0.15.0 (2021-04-19)
===================
- Add support for Python 3.9
- Abandon support for Python 3.5. If you still require support for Python 3.5, please use pytest-asyncio v0.14 or earlier.
- Set ``unused_tcp_port_factory`` fixture scope to 'session'.
  `#163 <https://github.com/pytest-dev/pytest-asyncio/pull/163>`_
- Properly close event loops when replacing them.
  `#208 <https://github.com/pytest-dev/pytest-asyncio/issues/208>`_

0.14.0 (2020-06-24)
===================
- Fix `#162 <https://github.com/pytest-dev/pytest-asyncio/issues/162>`_, and ``event_loop`` fixture behavior now is coherent on all scopes.
  `#164 <https://github.com/pytest-dev/pytest-asyncio/pull/164>`_

0.12.0 (2020-05-04)
===================
- Run the event loop fixture as soon as possible. This helps with fixtures that have an implicit dependency on the event loop.
  `#156 <https://github.com/pytest-dev/pytest-asyncio/pull/156>`_

0.11.0 (2020-04-20)
===================
- Test on 3.8, drop 3.3 and 3.4. Stick to 0.10 for these versions.
  `#152 <https://github.com/pytest-dev/pytest-asyncio/pull/152>`_
- Use the new Pytest 5.4.0 Function API. We therefore depend on pytest >= 5.4.0.
  `#142 <https://github.com/pytest-dev/pytest-asyncio/pull/142>`_
- Better ``pytest.skip`` support.
  `#126 <https://github.com/pytest-dev/pytest-asyncio/pull/126>`_

0.10.0 (2019-01-08)
====================
- ``pytest-asyncio`` integrates with `Hypothesis <https://hypothesis.readthedocs.io>`_
  to support ``@given`` on async test functions using ``asyncio``.
  `#102 <https://github.com/pytest-dev/pytest-asyncio/pull/102>`_
- Pytest 4.1 support.
  `#105 <https://github.com/pytest-dev/pytest-asyncio/pull/105>`_

0.9.0 (2018-07-28)
==================
- Python 3.7 support.
- Remove ``event_loop_process_pool`` fixture and
  ``pytest.mark.asyncio_process_pool`` marker (see
  https://bugs.python.org/issue34075 for deprecation and removal details)

0.8.0 (2017-09-23)
==================
- Improve integration with other packages (like aiohttp) with more careful event loop handling.
  `#64 <https://github.com/pytest-dev/pytest-asyncio/pull/64>`_

0.7.0 (2017-09-08)
==================
- Python versions pre-3.6 can use the async_generator library for async fixtures.
  `#62 <https://github.com/pytest-dev/pytest-asyncio/pull/62>`

0.6.0 (2017-05-28)
==================
- Support for Python versions pre-3.5 has been dropped.
- ``pytestmark`` now works on both module and class level.
- The ``forbid_global_loop`` parameter has been removed.
- Support for async and async gen fixtures has been added.
  `#45 <https://github.com/pytest-dev/pytest-asyncio/pull/45>`_
- The deprecation warning regarding ``asyncio.async()`` has been fixed.
  `#51 <https://github.com/pytest-dev/pytest-asyncio/pull/51>`_

0.5.0 (2016-09-07)
==================
- Introduced a changelog.
  `#31 <https://github.com/pytest-dev/pytest-asyncio/issues/31>`_
- The ``event_loop`` fixture is again responsible for closing itself.
  This makes the fixture slightly harder to correctly override, but enables
  other fixtures to depend on it correctly.
  `#30 <https://github.com/pytest-dev/pytest-asyncio/issues/30>`_
- Deal with the event loop policy by wrapping a special pytest hook,
  ``pytest_fixture_setup``. This allows setting the policy before fixtures
  dependent on the ``event_loop`` fixture run, thus allowing them to take
  advantage of the ``forbid_global_loop`` parameter. As a consequence of this,
  we now depend on pytest 3.0.
  `#29 <https://github.com/pytest-dev/pytest-asyncio/issues/29>`_

0.4.1 (2016-06-01)
==================
- Fix a bug preventing the propagation of exceptions from the plugin.
  `#25 <https://github.com/pytest-dev/pytest-asyncio/issues/25>`_

0.4.0 (2016-05-30)
==================
- Make ``event_loop`` fixtures simpler to override by closing them in the
  plugin, instead of directly in the fixture.
  `#21 <https://github.com/pytest-dev/pytest-asyncio/pull/21>`_
- Introduce the ``forbid_global_loop`` parameter.
  `#21 <https://github.com/pytest-dev/pytest-asyncio/pull/21>`_

0.3.0 (2015-12-19)
==================
- Support for Python 3.5 ``async``/``await`` syntax.
  `#17 <https://github.com/pytest-dev/pytest-asyncio/pull/17>`_

0.2.0 (2015-08-01)
==================
- ``unused_tcp_port_factory`` fixture.
  `#10 <https://github.com/pytest-dev/pytest-asyncio/issues/10>`_

0.1.1 (2015-04-23)
==================
Initial release.
