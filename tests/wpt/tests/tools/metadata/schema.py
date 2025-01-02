from typing import Any, Callable, cast, Dict, Sequence, Set, Type, TypeVar, Union

T = TypeVar("T")

def validate_dict(obj: Any, required_keys: Set[str] = set(), optional_keys: Set[str] = set()) -> None:
    """
    Validates the keys for a particular object
    This logic ensures:
    1. the obj is type dict
    2. That at a minimum the provided required_keys are present.
    Additionally, the logic checks for a set of optional_keys. With those two
    sets of keys, the logic will raise an error if there are extra keys in obj.
    :param obj: The object that will be checked.
    :param required_keys: Set of required keys that the obj should have.
    :param optional_keys: Set of optional keys that the obj should have.
    :return: `None` if obj does not have any extra keys.
    :raises ValueError: If there unexpected keys or missing required keys.
    """
    if not isinstance(obj, dict):
        raise ValueError(f"Object is not a dictionary. Input: {obj}")
    extra_keys = set(obj.keys()) - required_keys - optional_keys
    missing_required_keys = required_keys - set(obj.keys())
    if extra_keys:
        raise ValueError(f"Object contains invalid keys: {sorted(extra_keys)}")
    if missing_required_keys:
        raise ValueError(f"Object missing required keys: {sorted(missing_required_keys)}")


class SchemaValue():
    """
    Set of helpers to convert raw input into an expected value for a given schema
    """
    @staticmethod
    def from_dict(x: Any) -> Dict[str, Any]:
        if not isinstance(x, dict):
            raise ValueError(f"Input value {x} is not a dict")
        keys = x.keys()
        for key in keys:
            if not isinstance(key, str):
                raise ValueError(f"Input value {x} contains key {key} that is not a string")
        return cast(Dict[str, Any], x)


    @staticmethod
    def from_str(x: Any) -> str:
        if not isinstance(x, str):
            raise ValueError(f"Input value {x} is not a string")
        return x


    @staticmethod
    def from_none(x: Any) -> None:
        if x is not None:
            raise ValueError(f"Input value {x} is not none")
        return x


    @staticmethod
    def from_union(fs:
        Sequence[Union[
            Callable[[Any], Sequence[T]],
            Callable[[Any], T],
        ]],
            x: Any) -> Any:
        for f in fs:
            try:
                return f(x)
            except Exception:
                pass
        raise ValueError(f"Input value {x} does not fit one of the expected values for the union")


    @staticmethod
    def from_list(f: Callable[[Any], T], x: Any) -> Sequence[T]:
        if not isinstance(x, list):
            raise ValueError(f"Input value {x} is not a list")
        return [f(y) for y in x]


    @staticmethod
    def from_class(cls: Type[T]) -> Callable[[Any], T]:
        def class_converter(x: Any) -> T:
            try:
                # https://github.com/python/mypy/issues/10343
                return cls(x)  # type: ignore [call-arg]
            except Exception:
                raise ValueError(f"Input value {x} could not be converted to {cls}")
        return class_converter
