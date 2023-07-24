# mypy: allow-untyped-defs

import os
import pickle

here = os.path.abspath(os.path.dirname(__file__))

class BaseState:
    def __new__(cls, logger):
        rv = cls.load(logger)
        if rv is not None:
            logger.debug("Existing state found")
            return rv

        logger.debug("No existing state found")
        return super().__new__(cls)

    def __init__(self, logger):
        """Object containing state variables created when running Steps.

        Variables are set and get as attributes e.g. state_obj.spam = "eggs".

        :param parent: Parent State object or None if this is the root object.
        """

        if hasattr(self, "_data"):
            return

        self._data = [{}]
        self._logger = logger
        self._index = 0

    def __getstate__(self):
        rv = self.__dict__.copy()
        del rv["_logger"]
        return rv


    def push(self, init_values):
        """Push a new clean state dictionary

        :param init_values: List of variable names in the current state dict to copy
                            into the new state dict."""

        return StateContext(self, init_values)

    def is_empty(self):
        return len(self._data) == 1 and self._data[0] == {}

    def clear(self):
        """Remove all state and delete the stored copy."""
        self._data = [{}]

    def __setattr__(self, key, value):
        if key.startswith("_"):
            object.__setattr__(self, key, value)
        else:
            self._data[self._index][key] = value
            self.save()

    def __getattr__(self, key):
        if key.startswith("_"):
            raise AttributeError
        try:
            return self._data[self._index][key]
        except KeyError:
            raise AttributeError

    def __contains__(self, key):
        return key in self._data[self._index]

    def update(self, items):
        """Add a dictionary of {name: value} pairs to the state"""
        self._data[self._index].update(items)
        self.save()

    def keys(self):
        return self._data[self._index].keys()


    @classmethod
    def load(cls):
        raise NotImplementedError

    def save(self):
        raise NotImplementedError


class SavedState(BaseState):
    """On write the state is serialized to disk, such that it can be restored in
       the event that the program is interrupted before all steps are complete.
       Note that this only works well if the values are immutable; mutating an
       existing value will not cause the data to be serialized."""
    filename = os.path.join(here, ".wpt-update.lock")

    @classmethod
    def load(cls, logger):
        """Load saved state from a file"""
        try:
            if not os.path.isfile(cls.filename):
                return None
            with open(cls.filename, "rb") as f:
                try:
                    rv = pickle.load(f)
                    logger.debug(f"Loading data {rv._data!r}")
                    rv._logger = logger
                    rv._index = 0
                    return rv
                except EOFError:
                    logger.warning("Found empty state file")
        except OSError:
            logger.debug("IOError loading stored state")

    def save(self):
        """Write the state to disk"""
        with open(self.filename, "wb") as f:
            pickle.dump(self, f)

    def clear(self):
        super().clear()
        try:
            os.unlink(self.filename)
        except OSError:
            pass


class UnsavedState(BaseState):
    @classmethod
    def load(cls, logger):
        return None

    def save(self):
        return


class StateContext:
    def __init__(self, state, init_values):
        self.state = state
        self.init_values = init_values

    def __enter__(self):
        if len(self.state._data) == self.state._index + 1:
            # This is the case where there is no stored state
            new_state = {}
            for key in self.init_values:
                new_state[key] = self.state._data[self.state._index][key]
            self.state._data.append(new_state)
        self.state._index += 1
        self.state._logger.debug("Incremented index to %s" % self.state._index)

    def __exit__(self, *args, **kwargs):
        if len(self.state._data) > 1:
            assert self.state._index == len(self.state._data) - 1
            self.state._data.pop()
            self.state._index -= 1
            self.state._logger.debug("Decremented index to %s" % self.state._index)
            assert self.state._index >= 0
        else:
            raise ValueError("Tried to pop the top state")
