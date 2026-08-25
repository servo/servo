Release Notes
=============

This document contains release notes for Hyper-h2. In addition to the
:ref:`detailed-release-notes` found at the bottom of this document, this
document also includes a high-level prose overview of each major release after
1.0.0.

High Level Notes
----------------

3.0.0: 24 March 2017
~~~~~~~~~~~~~~~~~~~~

The Hyper-h2 team and the Hyper project are delighted to announce the release
of Hyper-h2 version 3.0.0! Unlike the really notable 2.0.0 release, this
release is proportionally quite small: however, it has the effect of removing a
lot of cruft and complexity that has built up in the codebase over the lifetime
of the v2 release series.

This release was motivated primarily by discovering that applications that
attempted to use both HTTP/1.1 and HTTP/2 using hyper-h2 would encounter
problems with cookies, because hyper-h2 did not join together cookie headers as
required by RFC 7540. Normally adding such behaviour would be a non-breaking
change, but we previously had no flags to prevent normalization of received
HTTP headers.

Because it makes no sense for the cookie to be split *by default*, we needed to
add a controlling flag and set it to true. The breaking nature of this change
is very subtle, and it's possible most users would never notice, but
nevertheless it *is* a breaking change and we need to treat it as such.

Happily, we can take this opportunity to finalise a bunch of deprecations we'd
made over the past year. The v2 release series was long-lived and successful,
having had a series of releases across the past year-and-a-bit, and the Hyper
team are very proud of it. However, it's time to open a new chapter, and remove
the deprecated code.

The past year has been enormously productive for the Hyper team. A total of 30
v2 releases were made, an enormous amount of work. A good number of people have
made their first contribution in this time, more than I can thank reasonably
without taking up an unreasonable amount of space in this document, so instead
I invite you to check out `our awesome contributor list`_.

We're looking forward to the next chapter in hyper-h2: it's been a fun ride so
far, and we hope even more of you come along and join in the fun over the next
year!

.. _our awesome contributor list: https://github.com/python-hyper/hyper-h2/graphs/contributors


2.0.0: 25 January 2016
~~~~~~~~~~~~~~~~~~~~~~

The Hyper-h2 team and the Hyper project are delighted to announce the release
of Hyper-h2 version 2.0.0! This is an enormous release that contains a gigantic
collection of new features and fixes, with the goal of making it easier than
ever to use Hyper-h2 to build a compliant HTTP/2 server or client.

An enormous chunk of this work has been focused on tighter enforcement of
restrictions in RFC 7540, ensuring that we correctly police the actions of
remote peers, and error appropriately when those peers violate the
specification. Several of these constitute breaking changes, because data that
was previously received and handled without obvious error now raises
``ProtocolError`` exceptions and causes the connection to be terminated.

Additionally, the public API was cleaned up and had several helper methods that
had been inavertently exposed removed from the public API. The team wants to
stress that while Hyper-h2 follows semantic versioning, the guarantees of
semver apply only to the public API as documented in :doc:`api`. Reducing the
surface area of these APIs makes it easier for us to continue to ensure that
the guarantees of semver are respected on our public API.

We also attempted to clear up some of the warts that had appeared in the API,
and add features that are helpful for implementing HTTP/2 endpoints. For
example, the :class:`H2Connection <h2.connection.H2Connection>` object now
exposes a method for generating the next stream ID that your client or server
can use to initiate a connection (:meth:`get_next_available_stream_id
<h2.connection.H2Connection.get_next_available_stream_id>`). We also removed
some needless return values that were guaranteed to return empty lists, which
were an attempt to make a forward-looking guarantee that was entirely unneeded.

Altogether, this has been an extremely productive period for Hyper-h2, and a
lot of great work has been done by the community. To that end, we'd also like
to extend a great thankyou to those contributors who made their first contribution
to the project between release 1.0.0 and 2.0.0. Many thanks to:
`Thomas Kriechbaumer`_, `Alex Chan`_, `Maximilian Hils`_, and `Glyph`_. For a
full historical list of contributors, see `contributors`_.

We're looking forward to the next few months of Python HTTP/2 work, and hoping
that you'll find lots of excellent HTTP/2 applications to build with Hyper-h2!


.. _Thomas Kriechbaumer: https://github.com/Kriechi
.. _Alex Chan: https://github.com/alexwlchan
.. _Maximilian Hils: https://github.com/mhils
.. _Glyph: https://github.com/glyph
.. _contributors: https://github.com/python-hyper/hyper-h2/blob/b14817b79c7bb1661e1aa84ef7920c009ef1e75b/CONTRIBUTORS.rst


.. _detailed-release-notes:
.. include:: ../../CHANGELOG.rst
