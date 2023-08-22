# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import inspect

convertor_registry = {}
missing = object()
no_default = object()


class log_action(object):
    def __init__(self, *args):
        self.args = {}

        self.args_no_default = []
        self.args_with_default = []
        self.optional_args = set()

        # These are the required fields in a log message that usually aren't
        # supplied by the caller, but can be in the case of log_raw
        self.default_args = [
            Unicode("action"),
            Int("time"),
            Unicode("thread"),
            Int("pid", default=None),
            Unicode("source"),
            Unicode("component"),
        ]

        for arg in args:
            if arg.default is no_default:
                self.args_no_default.append(arg.name)
            else:
                self.args_with_default.append(arg.name)

            if arg.optional:
                self.optional_args.add(arg.name)

            if arg.name in self.args:
                raise ValueError("Repeated argument name %s" % arg.name)

            self.args[arg.name] = arg

        for extra in self.default_args:
            self.args[extra.name] = extra

    def __call__(self, f):
        convertor_registry[f.__name__] = self
        converter = self

        def inner(self, *args, **kwargs):
            data = converter.convert(*args, **kwargs)
            return f(self, data)

        if hasattr(f, "__doc__"):
            setattr(inner, "__doc__", f.__doc__)

        return inner

    def convert(self, *args, **kwargs):
        data = {}
        values = {}
        values.update(kwargs)

        positional_no_default = [
            item for item in self.args_no_default if item not in values
        ]

        num_no_default = len(positional_no_default)

        if len(args) < num_no_default:
            raise TypeError("Too few arguments")

        if len(args) > num_no_default + len(self.args_with_default):
            raise TypeError("Too many arguments")

        for i, name in enumerate(positional_no_default):
            values[name] = args[i]

        positional_with_default = [
            self.args_with_default[i] for i in range(len(args) - num_no_default)
        ]

        for i, name in enumerate(positional_with_default):
            if name in values:
                raise TypeError("Argument %s specified twice" % name)
            values[name] = args[i + num_no_default]

        # Fill in missing arguments
        for name in self.args_with_default:
            if name not in values:
                values[name] = self.args[name].default

        for key, value in values.items():
            if key in self.args:
                out_value = self.args[key](value)
                if out_value is not missing:
                    if key in self.optional_args and value == self.args[key].default:
                        pass
                    else:
                        data[key] = out_value
            else:
                raise TypeError("Unrecognised argument %s" % key)

        return data

    def convert_known(self, **kwargs):
        known_kwargs = {
            name: value for name, value in kwargs.items() if name in self.args
        }
        return self.convert(**known_kwargs)


class DataType(object):
    def __init__(self, name, default=no_default, optional=False):
        self.name = name
        self.default = default

        if default is no_default and optional is not False:
            raise ValueError("optional arguments require a default value")

        self.optional = optional

    def __call__(self, value):
        if value == self.default:
            if self.optional:
                return missing
            return self.default

        try:
            return self.convert(value)
        except Exception:
            raise ValueError(
                "Failed to convert value %s of type %s for field %s to type %s"
                % (value, type(value).__name__, self.name, self.__class__.__name__)
            )


class ContainerType(DataType):
    """A DataType that contains other DataTypes.

    ContainerTypes must specify which other DataType they will contain. ContainerTypes
    may contain other ContainerTypes.

    Some examples:

        List(Int, 'numbers')
        Tuple((Unicode, Int, Any), 'things')
        Dict(Unicode, 'config')
        Dict({TestId: Status}, 'results')
        Dict(List(Unicode), 'stuff')
    """

    def __init__(self, item_type, name=None, **kwargs):
        DataType.__init__(self, name, **kwargs)
        self.item_type = self._format_item_type(item_type)

    def _format_item_type(self, item_type):
        if inspect.isclass(item_type):
            return item_type(None)
        return item_type


class Unicode(DataType):
    def convert(self, data):
        if isinstance(data, str):
            return data
        return str(data)


class TestId(DataType):
    def convert(self, data):
        if isinstance(data, str):
            return data
        elif isinstance(data, bytes):
            return data.decode("utf-8", "replace")
        elif isinstance(data, (tuple, list)):
            # This is really a bit of a hack; should really split out convertors from the
            # fields they operate on
            func = Unicode(None).convert
            return tuple(func(item) for item in data)
        else:
            raise ValueError


class Status(DataType):
    allowed = [
        "PASS",
        "FAIL",
        "OK",
        "ERROR",
        "TIMEOUT",
        "CRASH",
        "ASSERT",
        "PRECONDITION_FAILED",
        "SKIP",
    ]

    def convert(self, data):
        value = data.upper()
        if value not in self.allowed:
            raise ValueError
        return value


class SubStatus(Status):
    allowed = [
        "PASS",
        "FAIL",
        "ERROR",
        "TIMEOUT",
        "ASSERT",
        "PRECONDITION_FAILED",
        "NOTRUN",
        "SKIP",
    ]


class Dict(ContainerType):
    def _format_item_type(self, item_type):
        superfmt = super(Dict, self)._format_item_type

        if isinstance(item_type, dict):
            if len(item_type) != 1:
                raise ValueError(
                    "Dict item type specifier must contain a single entry."
                )
            key_type, value_type = list(item_type.items())[0]
            return superfmt(key_type), superfmt(value_type)
        return Any(None), superfmt(item_type)

    def convert(self, data):
        key_type, value_type = self.item_type
        return {
            key_type.convert(k): value_type.convert(v) for k, v in dict(data).items()
        }


class List(ContainerType):
    def convert(self, data):
        # while dicts and strings _can_ be cast to lists,
        # doing so is likely not intentional behaviour
        if isinstance(data, (str, dict)):
            raise ValueError("Expected list but got %s" % type(data))
        return [self.item_type.convert(item) for item in data]


class TestList(DataType):
    """A TestList is a list of tests that can be either keyed by a group name,
    or specified as a flat list.
    """

    def convert(self, data):
        if isinstance(data, (list, tuple)):
            data = {"default": data}
        return Dict({Unicode: List(Unicode)}).convert(data)


class Int(DataType):
    def convert(self, data):
        return int(data)


class Any(DataType):
    def convert(self, data):
        return data


class Boolean(DataType):
    def convert(self, data):
        return bool(data)


class Tuple(ContainerType):
    def _format_item_type(self, item_type):
        superfmt = super(Tuple, self)._format_item_type

        if isinstance(item_type, (tuple, list)):
            return [superfmt(t) for t in item_type]
        return (superfmt(item_type),)

    def convert(self, data):
        if len(data) != len(self.item_type):
            raise ValueError(
                "Expected %i items got %i" % (len(self.item_type), len(data))
            )
        return tuple(
            item_type.convert(value) for item_type, value in zip(self.item_type, data)
        )


class Nullable(ContainerType):
    def convert(self, data):
        if data is None:
            return data
        return self.item_type.convert(data)
