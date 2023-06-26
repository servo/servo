iniconfig: brain-dead simple parsing of ini files
=======================================================

iniconfig is a small and simple INI-file parser module
having a unique set of features:

* tested against Python2.4 across to Python3.2, Jython, PyPy
* maintains order of sections and entries
* supports multi-line values with or without line-continuations
* supports "#" comments everywhere
* raises errors with proper line-numbers
* no bells and whistles like automatic substitutions
* iniconfig raises an Error if two sections have the same name.

If you encounter issues or have feature wishes please report them to:

    http://github.com/RonnyPfannschmidt/iniconfig/issues

Basic Example
===================================

If you have an ini file like this::

    # content of example.ini
    [section1] # comment
    name1=value1  # comment
    name1b=value1,value2  # comment

    [section2]
    name2=
        line1
        line2

then you can do::

    >>> import iniconfig
    >>> ini = iniconfig.IniConfig("example.ini")
    >>> ini['section1']['name1'] # raises KeyError if not exists
    'value1'
    >>> ini.get('section1', 'name1b', [], lambda x: x.split(","))
    ['value1', 'value2']
    >>> ini.get('section1', 'notexist', [], lambda x: x.split(","))
    []
    >>> [x.name for x in list(ini)]
    ['section1', 'section2']
    >>> list(list(ini)[0].items())
    [('name1', 'value1'), ('name1b', 'value1,value2')]
    >>> 'section1' in ini
    True
    >>> 'inexistendsection' in ini
    False
