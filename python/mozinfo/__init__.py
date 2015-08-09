# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

"""
interface to transform introspected system information to a format palatable to
Mozilla

Module variables:

.. attribute:: bits

   32 or 64

.. attribute:: isBsd

   Returns ``True`` if the operating system is BSD

.. attribute:: isLinux

   Returns ``True`` if the operating system is Linux

.. attribute:: isMac

   Returns ``True`` if the operating system is Mac

.. attribute:: isWin

   Returns ``True`` if the operating system is Windows

.. attribute:: os

   Operating system [``'win'``, ``'mac'``, ``'linux'``, ...]

.. attribute:: processor

   Processor architecture [``'x86'``, ``'x86_64'``, ``'ppc'``, ...]

.. attribute:: version

   Operating system version string. For windows, the service pack information is also included

.. attribute:: info

   Returns information identifying the current system.

   * :attr:`bits`
   * :attr:`os`
   * :attr:`processor`
   * :attr:`version`

"""

import mozinfo
from mozinfo import *
__all__ = mozinfo.__all__
