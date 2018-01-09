import py
import sys


@py.test.mark.parametrize('name', [x for x in dir(py) if x[0] != '_'])
def test_dir(name):
    obj = getattr(py, name)
    if hasattr(obj, '__map__'):  # isinstance(obj, Module):
        keys = dir(obj)
        assert len(keys) > 0
        print (obj.__map__)
        for name in list(obj.__map__):
            assert hasattr(obj, name), (obj, name)


def test_virtual_module_identity():
    from py import path as path1
    from py import path as path2
    assert path1 is path2
    from py.path import local as local1
    from py.path import local as local2
    assert local1 is local2


def test_importall():
    base = py._pydir
    nodirs = [
    ]
    if sys.version_info >= (3, 0):
        nodirs.append(base.join('_code', '_assertionold.py'))
    else:
        nodirs.append(base.join('_code', '_assertionnew.py'))

    def recurse(p):
        return p.check(dotfile=0) and p.basename != "attic"

    for p in base.visit('*.py', recurse):
        if p.basename == '__init__.py':
            continue
        relpath = p.new(ext='').relto(base)
        if base.sep in relpath:  # not py/*.py itself
            for x in nodirs:
                if p == x or p.relto(x):
                    break
            else:
                relpath = relpath.replace(base.sep, '.')
                modpath = 'py.%s' % relpath
                try:
                    check_import(modpath)
                except py.test.skip.Exception:
                    pass


def check_import(modpath):
    py.builtin.print_("checking import", modpath)
    assert __import__(modpath)


def test_star_import():
    exec("from py import *")


def test_all_resolves():
    seen = py.builtin.set([py])
    lastlength = None
    while len(seen) != lastlength:
        lastlength = len(seen)
        for item in py.builtin.frozenset(seen):
            for value in item.__dict__.values():
                if isinstance(value, type(py.test)):
                    seen.add(value)
