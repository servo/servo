python testing sprint June 20th-26th 2016
======================================================

.. image:: ../img/freiburg2.jpg
   :width: 400

The pytest core group is heading towards the biggest sprint
in its history, to take place in the black forest town Freiburg
in Germany.  As of February 2016 we have started a `funding
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

Here are preliminary participants who said they are likely to come,
given some expenses funding::

    Anatoly Bubenkoff, Netherlands
    Andreas Pelme, Personalkollen, Sweden
    Anthony Wang, Splunk, US
    Brianna Laugher, Australia
    Bruno Oliveira, Brazil
    Danielle Jenkins, Splunk, US
    Dave Hunt, UK
    Florian Bruhin, Switzerland
    Floris Bruynooghe, Cobe.io, UK
    Holger Krekel, merlinux, Germany
    Oliver Bestwalter, Avira, Germany
    Omar Kohl, Germany
    Raphael Pierzina, FanDuel, UK
    Tom Viner, UK

    <your name here?>

Other contributors and experienced newcomers are invited to join as well
but please send a mail to the pytest-dev mailing list if you intend to
do so somewhat soon, also how much funding you need if so.  And if you
are working for a company and using pytest heavily you are welcome to
join and we encourage your company to provide some funding for the
sprint.  They may see it, and rightfully so, as a very cheap and deep
training which brings you together with the experts in the field :)


Sprint organisation, schedule
-------------------------------

tentative schedule:

- 19/20th arrival in Freiburg
- 20th social get together, initial hacking
- 21/22th full sprint days
- 23rd break day, hiking
- 24/25th full sprint days
- 26th departure

We might adjust according to weather to make sure that if
we do some hiking or excursion we'll have good weather.
Freiburg is one of the sunniest places in Germany so
it shouldn't be too much of a constraint.


Accomodation
----------------

We'll see to arrange for renting a flat with multiple
beds/rooms.  Hotels are usually below 100 per night.
The earlier we book the better.

Money / funding
---------------

The Indiegogo campaign asks for 11000 USD which should cover
the costs for flights and accomodation, renting a sprint place
and maybe a bit of food as well.

If your organisation wants to support the sprint but prefers
to give money according to an invoice, get in contact with
holger at http://merlinux.eu who can invoice your organisation
properly.

If we have excess money we'll use for further sprint/travel
funding for pytest/tox contributors.
