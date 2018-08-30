# -*- coding: utf-8 -*-

#    **************
#    * Slugid API *
#    **************
#
#       @)@)
#       _|_|                                      (   )
#     _(___,`\      _,--------------._          (( /`, ))
#     `==`   `*-_,'          O        `~._   ( ( _/  |  ) )
#      `,    :         o              }   `~._.~`  * ',
#        \      -         _      O              -    ,'
#        |  ;      -          -      "      ;     o  /
#        |      O                        o        ,-`
#        \          _,-:""""""'`:-._    -  .   O /
#         `""""""~'`                `._      _,-`
#                                      """"""

"""
SlugID: Base 64 encoded v4 UUIDs
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Usage:

   >>> import slugid
   >>> s = slugid.nice()
   >>> s
   eWIgwMgxSfeXQ36iPbOxiQ
   >>> u = slugid.decode(s)
   >>> u
   UUID('796220c0-c831-49f7-9743-7ea23db3b189')
   >>> slugid.encode(u)
   eWIgwMgxSfeXQ36iPbOxiQ
   >>> slugid.v4()
   -9OpXaCORAaFh4sJRk7PUA
"""
from .slugid import decode, encode, nice, v4

__title__ = 'slugid'
__version__ = '1.0.7'
__author__ = 'Peter Moore'
__license__ = 'MPL 2.0'
__all__ = [
     'decode',
     'encode',
     'nice',
     'v4',
]
