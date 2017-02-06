from __future__ import with_statement
import py, pytest

# test for _argcomplete but not specific for any application

def equal_with_bash(prefix, ffc, fc, out=None):
    res = ffc(prefix)
    res_bash = set(fc(prefix))
    retval = set(res) == res_bash
    if out:
        out.write('equal_with_bash %s %s\n' % (retval, res))
        if not retval:
            out.write(' python - bash: %s\n' % (set(res) - res_bash))
            out.write(' bash - python: %s\n' % (res_bash - set(res)))
    return retval

# copied from argcomplete.completers as import from there
# also pulls in argcomplete.__init__ which opens filedescriptor 9
# this gives an IOError at the end of testrun
def _wrapcall(*args, **kargs):
    try:
        if py.std.sys.version_info > (2,7):
            return py.std.subprocess.check_output(*args,**kargs).decode().splitlines()
        if 'stdout' in kargs:
            raise ValueError('stdout argument not allowed, it will be overridden.')
        process = py.std.subprocess.Popen(
            stdout=py.std.subprocess.PIPE, *args, **kargs)
        output, unused_err = process.communicate()
        retcode = process.poll()
        if retcode:
            cmd = kargs.get("args")
            if cmd is None:
                cmd = args[0]
            raise py.std.subprocess.CalledProcessError(retcode, cmd)
        return output.decode().splitlines()
    except py.std.subprocess.CalledProcessError:
        return []

class FilesCompleter(object):
    'File completer class, optionally takes a list of allowed extensions'
    def __init__(self,allowednames=(),directories=True):
        # Fix if someone passes in a string instead of a list
        if type(allowednames) is str:
            allowednames = [allowednames]

        self.allowednames = [x.lstrip('*').lstrip('.') for x in allowednames]
        self.directories = directories

    def __call__(self, prefix, **kwargs):
        completion = []
        if self.allowednames:
            if self.directories:
                files = _wrapcall(['bash','-c',
                    "compgen -A directory -- '{p}'".format(p=prefix)])
                completion += [ f + '/' for f in files]
            for x in self.allowednames:
                completion += _wrapcall(['bash', '-c',
                    "compgen -A file -X '!*.{0}' -- '{p}'".format(x,p=prefix)])
        else:
            completion += _wrapcall(['bash', '-c',
                "compgen -A file -- '{p}'".format(p=prefix)])

            anticomp = _wrapcall(['bash', '-c',
                "compgen -A directory -- '{p}'".format(p=prefix)])

            completion = list( set(completion) - set(anticomp))

            if self.directories:
                completion += [f + '/' for f in anticomp]
        return completion

class TestArgComplete:
    @pytest.mark.skipif("sys.platform in ('win32', 'darwin')")
    def test_compare_with_compgen(self):
        from _pytest._argcomplete import FastFilesCompleter
        ffc = FastFilesCompleter()
        fc = FilesCompleter()
        for x in '/ /d /data qqq'.split():
            assert equal_with_bash(x, ffc, fc, out=py.std.sys.stdout)

    @pytest.mark.skipif("sys.platform in ('win32', 'darwin')")
    def test_remove_dir_prefix(self):
        """this is not compatible with compgen but it is with bash itself:
        ls /usr/<TAB>
        """
        from _pytest._argcomplete import FastFilesCompleter
        ffc = FastFilesCompleter()
        fc = FilesCompleter()
        for x in '/usr/'.split():
            assert not equal_with_bash(x, ffc, fc, out=py.std.sys.stdout)
