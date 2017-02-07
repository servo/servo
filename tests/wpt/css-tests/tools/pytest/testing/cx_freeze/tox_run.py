"""
Called by tox.ini: uses the generated executable to run the tests in ./tests/
directory.

.. note:: somehow calling "build/runtests_script" directly from tox doesn't
          seem to work (at least on Windows).
"""
if __name__ == '__main__':
    import os
    import sys

    executable = os.path.join(os.getcwd(), 'build', 'runtests_script')
    if sys.platform.startswith('win'):
        executable += '.exe'
    sys.exit(os.system('%s tests' % executable))