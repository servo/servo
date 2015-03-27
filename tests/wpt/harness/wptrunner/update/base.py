# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

exit_unclean = object()
exit_clean = object()


class Step(object):
    provides = []

    def __init__(self, logger):
        self.logger = logger

    def run(self, step_index, state):
        """Base class for state-creating steps.

        When a Step is run() the current state is checked to see
        if the state from this step has already been created. If it
        has the restore() method is invoked. Otherwise the create()
        method is invoked with the state object. This is expected to
        add items with all the keys in __class__.provides to the state
        object.
        """

        name = self.__class__.__name__

        try:
            stored_step = state.steps[step_index]
        except IndexError:
            stored_step = None

        if stored_step == name:
            self.restore(state)
        elif stored_step is None:
            self.create(state)
            assert set(self.provides).issubset(set(state.keys()))
            state.steps = state.steps + [name]
        else:
            raise ValueError("Expected a %s step, got a %s step" % (name, stored_step))

    def create(self, data):
        raise NotImplementedError

    def restore(self, state):
        self.logger.debug("Step %s using stored state" % (self.__class__.__name__,))
        for key in self.provides:
            assert key in state


class StepRunner(object):
    steps = []

    def __init__(self, logger, state):
        """Class that runs a specified series of Steps with a common State"""
        self.state = state
        self.logger = logger
        if "steps" not in state:
            state.steps = []

    def run(self):
        rv = None
        for step_index, step in enumerate(self.steps):
            self.logger.debug("Starting step %s" % step.__name__)
            rv = step(self.logger).run(step_index, self.state)
            if rv in (exit_clean, exit_unclean):
                break

        return rv
