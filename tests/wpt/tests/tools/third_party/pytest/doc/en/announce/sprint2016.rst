python testing sprint June 20th-26th 2016
======================================================

.. image:: ../img/freiburg2.jpg
   :width: 400

The pytest core group held the biggest sprint
in its history in June 2016, taking place in the black forest town Freiburg
in Germany.  In February 2016 we started a `funding
campaign on Indiegogo to cover expenses
<http://igg.me/at/pytest-sprint/x/4034848>`_ The page also mentions
some preliminary topics:

- improving pytest-xdist test scheduling to take into account
  fixture setups and explicit user hints.

- provide info on fixture dependencies during --collect-only

- tying pytest-xdist to tox so that you can do "py.test -e py34"
  to run tests in a particular tox-managed virtualenv.  Also
  look into making pytest-xdist use tox environments on
  remote ssh-sides so that remote dependency management becomes
  easier.

- refactoring the fixture system so more people understand it :)

- integrating PyUnit setup methods as autouse fixtures.
  possibly adding ways to influence ordering of same-scoped
  fixtures (so you can make a choice of which fixtures come
  before others)

- fixing bugs and issues from the tracker, really an endless source :)


Participants
--------------

Over 20 participants took part from 4 continents, including employees
from Splunk, Personalkollen, Cobe.io, FanDuel and Dolby. Some newcomers
mixed with developers who have worked on pytest since its beginning, and
of course everyone in between.


Sprint organisation, schedule
-------------------------------

People arrived in Freiburg on the 19th, with sprint development taking
place on 20th, 21st, 22nd, 24th and 25th. On the 23rd we took a break
day for some hot hiking in the Black Forest.

Sprint activity was organised heavily around pairing, with plenty of group
discussions to take advantage of the high bandwidth, and lightning talks
as well.


Money / funding
---------------


The Indiegogo campaign aimed for 11000 USD and in the end raised over
12000, to reimburse travel costs, pay for a sprint venue and catering.

Excess money is reserved for further sprint/travel funding for pytest/tox
contributors.
