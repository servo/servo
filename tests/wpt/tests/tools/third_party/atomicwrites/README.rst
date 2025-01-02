===================
python-atomicwrites
===================

.. image:: https://travis-ci.org/untitaker/python-atomicwrites.svg?branch=master
    :target: https://travis-ci.org/untitaker/python-atomicwrites

.. image:: https://ci.appveyor.com/api/projects/status/vadc4le3c27to59x/branch/master?svg=true
   :target: https://ci.appveyor.com/project/untitaker/python-atomicwrites/branch/master

Atomic file writes.

.. code-block:: python

    from atomicwrites import atomic_write

    with atomic_write('foo.txt', overwrite=True) as f:
        f.write('Hello world.')
        # "foo.txt" doesn't exist yet.

    # Now it does.


Features that distinguish it from other similar libraries (see `Alternatives and Credit`_):

- Race-free assertion that the target file doesn't yet exist. This can be
  controlled with the ``overwrite`` parameter.

- Windows support, although not well-tested. The MSDN resources are not very
  explicit about which operations are atomic.

- Simple high-level API that wraps a very flexible class-based API.

- Consistent error handling across platforms.


How it works
============

It uses a temporary file in the same directory as the given path. This ensures
that the temporary file resides on the same filesystem.

The temporary file will then be atomically moved to the target location: On
POSIX, it will use ``rename`` if files should be overwritten, otherwise a
combination of ``link`` and ``unlink``. On Windows, it uses MoveFileEx_ through
stdlib's ``ctypes`` with the appropriate flags.

Note that with ``link`` and ``unlink``, there's a timewindow where the file
might be available under two entries in the filesystem: The name of the
temporary file, and the name of the target file.

Also note that the permissions of the target file may change this way. In some
situations a ``chmod`` can be issued without any concurrency problems, but
since that is not always the case, this library doesn't do it by itself.

.. _MoveFileEx: https://msdn.microsoft.com/en-us/library/windows/desktop/aa365240%28v=vs.85%29.aspx

fsync
-----

On POSIX, ``fsync`` is invoked on the temporary file after it is written (to
flush file content and metadata), and on the parent directory after the file is
moved (to flush filename).

``fsync`` does not take care of disks' internal buffers, but there don't seem
to be any standard POSIX APIs for that. On OS X, ``fcntl`` is used with
``F_FULLFSYNC`` instead of ``fsync`` for that reason.

On Windows, `_commit <https://msdn.microsoft.com/en-us/library/17618685.aspx>`_
is used, but there are no guarantees about disk internal buffers.

Alternatives and Credit
=======================

Atomicwrites is directly inspired by the following libraries (and shares a
minimal amount of code):

- The Trac project's `utility functions
  <http://www.edgewall.org/docs/tags-trac-0.11.7/epydoc/trac.util-pysrc.html>`_,
  also used in `Werkzeug <http://werkzeug.pocoo.org/>`_ and
  `mitsuhiko/python-atomicfile
  <https://github.com/mitsuhiko/python-atomicfile>`_. The idea to use
  ``ctypes`` instead of ``PyWin32`` originated there.

- `abarnert/fatomic <https://github.com/abarnert/fatomic>`_. Windows support
  (based on ``PyWin32``) was originally taken from there.

Other alternatives to atomicwrites include:

- `sashka/atomicfile <https://github.com/sashka/atomicfile>`_. Originally I
  considered using that, but at the time it was lacking a lot of features I
  needed (Windows support, overwrite-parameter, overriding behavior through
  subclassing).

- The `Boltons library collection <https://github.com/mahmoud/boltons>`_
  features a class for atomic file writes, which seems to have a very similar
  ``overwrite`` parameter. It is lacking Windows support though.

License
=======

Licensed under the MIT, see ``LICENSE``.
