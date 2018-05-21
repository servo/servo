# -*- coding: utf-8 -*-
import os
import pytest


def test_nothing_logged(testdir):
    testdir.makepyfile('''
        import sys

        def test_foo():
            sys.stdout.write('text going to stdout')
            sys.stderr.write('text going to stderr')
            assert False
        ''')
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines(['*- Captured stdout call -*',
                                 'text going to stdout'])
    result.stdout.fnmatch_lines(['*- Captured stderr call -*',
                                 'text going to stderr'])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(['*- Captured *log call -*'])


def test_messages_logged(testdir):
    testdir.makepyfile('''
        import sys
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            sys.stdout.write('text going to stdout')
            sys.stderr.write('text going to stderr')
            logger.info('text going to logger')
            assert False
        ''')
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines(['*- Captured *log call -*',
                                 '*text going to logger*'])
    result.stdout.fnmatch_lines(['*- Captured stdout call -*',
                                 'text going to stdout'])
    result.stdout.fnmatch_lines(['*- Captured stderr call -*',
                                 'text going to stderr'])


def test_setup_logging(testdir):
    testdir.makepyfile('''
        import logging

        logger = logging.getLogger(__name__)

        def setup_function(function):
            logger.info('text going to logger from setup')

        def test_foo():
            logger.info('text going to logger from call')
            assert False
        ''')
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines(['*- Captured *log setup -*',
                                 '*text going to logger from setup*',
                                 '*- Captured *log call -*',
                                 '*text going to logger from call*'])


def test_teardown_logging(testdir):
    testdir.makepyfile('''
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            logger.info('text going to logger from call')

        def teardown_function(function):
            logger.info('text going to logger from teardown')
            assert False
        ''')
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines(['*- Captured *log call -*',
                                 '*text going to logger from call*',
                                 '*- Captured *log teardown -*',
                                 '*text going to logger from teardown*'])


def test_disable_log_capturing(testdir):
    testdir.makepyfile('''
        import sys
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            sys.stdout.write('text going to stdout')
            logger.warning('catch me if you can!')
            sys.stderr.write('text going to stderr')
            assert False
        ''')
    result = testdir.runpytest('--no-print-logs')
    print(result.stdout)
    assert result.ret == 1
    result.stdout.fnmatch_lines(['*- Captured stdout call -*',
                                 'text going to stdout'])
    result.stdout.fnmatch_lines(['*- Captured stderr call -*',
                                 'text going to stderr'])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(['*- Captured *log call -*'])


def test_disable_log_capturing_ini(testdir):
    testdir.makeini(
        '''
        [pytest]
        log_print=False
        '''
    )
    testdir.makepyfile('''
        import sys
        import logging

        logger = logging.getLogger(__name__)

        def test_foo():
            sys.stdout.write('text going to stdout')
            logger.warning('catch me if you can!')
            sys.stderr.write('text going to stderr')
            assert False
        ''')
    result = testdir.runpytest()
    print(result.stdout)
    assert result.ret == 1
    result.stdout.fnmatch_lines(['*- Captured stdout call -*',
                                 'text going to stdout'])
    result.stdout.fnmatch_lines(['*- Captured stderr call -*',
                                 'text going to stderr'])
    with pytest.raises(pytest.fail.Exception):
        result.stdout.fnmatch_lines(['*- Captured *log call -*'])


def test_log_cli_default_level(testdir):
    # Default log file level
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_cli(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_cli_handler.level == logging.WARNING
            logging.getLogger('catchlog').info("This log message won't be shown")
            logging.getLogger('catchlog').warning("This log message will be shown")
            print('PASSED')
    ''')

    result = testdir.runpytest('-s')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_cli_default_level.py PASSED',
    ])
    result.stderr.fnmatch_lines([
        "* This log message will be shown"
    ])
    for line in result.errlines:
        try:
            assert "This log message won't be shown" in line
            pytest.fail("A log message was shown and it shouldn't have been")
        except AssertionError:
            continue

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


def test_log_cli_level(testdir):
    # Default log file level
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_cli(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_cli_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    ''')

    result = testdir.runpytest('-s', '--log-cli-level=INFO')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_cli_level.py PASSED',
    ])
    result.stderr.fnmatch_lines([
        "* This log message will be shown"
    ])
    for line in result.errlines:
        try:
            assert "This log message won't be shown" in line
            pytest.fail("A log message was shown and it shouldn't have been")
        except AssertionError:
            continue

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0

    result = testdir.runpytest('-s', '--log-level=INFO')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_cli_level.py PASSED',
    ])
    result.stderr.fnmatch_lines([
        "* This log message will be shown"
    ])
    for line in result.errlines:
        try:
            assert "This log message won't be shown" in line
            pytest.fail("A log message was shown and it shouldn't have been")
        except AssertionError:
            continue

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


def test_log_cli_ini_level(testdir):
    testdir.makeini(
        """
        [pytest]
        log_cli_level = INFO
        """)
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_cli(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_cli_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    ''')

    result = testdir.runpytest('-s')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_cli_ini_level.py PASSED',
    ])
    result.stderr.fnmatch_lines([
        "* This log message will be shown"
    ])
    for line in result.errlines:
        try:
            assert "This log message won't be shown" in line
            pytest.fail("A log message was shown and it shouldn't have been")
        except AssertionError:
            continue

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0


def test_log_file_cli(testdir):
    # Default log file level
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.WARNING
            logging.getLogger('catchlog').info("This log message won't be shown")
            logging.getLogger('catchlog').warning("This log message will be shown")
            print('PASSED')
    ''')

    log_file = testdir.tmpdir.join('pytest.log').strpath

    result = testdir.runpytest('-s', '--log-file={0}'.format(log_file))

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_file_cli.py PASSED',
    ])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_file_cli_level(testdir):
    # Default log file level
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    ''')

    log_file = testdir.tmpdir.join('pytest.log').strpath

    result = testdir.runpytest('-s',
                               '--log-file={0}'.format(log_file),
                               '--log-file-level=INFO')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_file_cli_level.py PASSED',
    ])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_file_ini(testdir):
    log_file = testdir.tmpdir.join('pytest.log').strpath

    testdir.makeini(
        """
        [pytest]
        log_file={0}
        """.format(log_file))
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.WARNING
            logging.getLogger('catchlog').info("This log message won't be shown")
            logging.getLogger('catchlog').warning("This log message will be shown")
            print('PASSED')
    ''')

    result = testdir.runpytest('-s')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_file_ini.py PASSED',
    ])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents


def test_log_file_ini_level(testdir):
    log_file = testdir.tmpdir.join('pytest.log').strpath

    testdir.makeini(
        """
        [pytest]
        log_file={0}
        log_file_level = INFO
        """.format(log_file))
    testdir.makepyfile('''
        import pytest
        import logging
        def test_log_file(request):
            plugin = request.config.pluginmanager.getplugin('logging-plugin')
            assert plugin.log_file_handler.level == logging.INFO
            logging.getLogger('catchlog').debug("This log message won't be shown")
            logging.getLogger('catchlog').info("This log message will be shown")
            print('PASSED')
    ''')

    result = testdir.runpytest('-s')

    # fnmatch_lines does an assertion internally
    result.stdout.fnmatch_lines([
        'test_log_file_ini_level.py PASSED',
    ])

    # make sure that that we get a '0' exit code for the testsuite
    assert result.ret == 0
    assert os.path.isfile(log_file)
    with open(log_file) as rfh:
        contents = rfh.read()
        assert "This log message will be shown" in contents
        assert "This log message won't be shown" not in contents
