import asyncio
import functools
from typing import (
    Any,
    Callable,
    Optional,
    Mapping,
    MutableMapping,
    TYPE_CHECKING,
)

from ..undefined import UNDEFINED

if TYPE_CHECKING:
    from ..client import BidiSession


class command:
    """Decorator for implementing bidi commands.

    Implementing a command involves specifying an async function that
    builds the parameters to the command. The decorator arranges those
    parameters to be turned into a send_command call, using the class
    and method names to determine the method in the call.

    Commands decorated in this way don't return a future, but await
    the actual response. In some cases it can be useful to
    post-process this response before returning it to the client. This
    can be done by specifying a second decorated method like
    @command_name.result. That method will then be called once the
    result of the original command is known, and the return value of
    the method used as the response of the command. If this method
    is specified, the `raw_result` parameter of the command can be set
    to `True` to get the result without post-processing.

    So for an example, if we had a command test.testMethod, which
    returned a result which we want to convert to a TestResult type,
    the implementation might look like:

    class Test(BidiModule):
        @command
        def test_method(self, test_data=None):
            return {"testData": test_data}

       @test_method.result
       def convert_test_method_result(self, result):
           return TestData(**result)
    """

    def __init__(self, fn: Callable[..., Mapping[str, Any]]):
        self.params_fn = fn
        self.result_fn: Optional[Callable[..., Any]] = None

    def result(self, fn: Callable[[Any, MutableMapping[str, Any]],
                                  Any]) -> None:
        self.result_fn = fn

    def __set_name__(self, owner: Any, name: str) -> None:
        # This is called when the class is created
        # see https://docs.python.org/3/reference/datamodel.html#object.__set_name__
        params_fn = self.params_fn
        result_fn = self.result_fn

        @functools.wraps(params_fn)
        async def inner(self: Any, **kwargs: Any) -> Any:
            raw_result = kwargs.pop("raw_result", False)
            params = remove_undefined(params_fn(self, **kwargs))

            # Convert the classname and the method name to a bidi command name
            mod_name = owner.__name__[0].lower() + owner.__name__[1:]
            if hasattr(owner, "prefix"):
                mod_name = f"{owner.prefix}:{mod_name}"
            cmd_name = f"{mod_name}.{to_camelcase(name)}"

            future = await self.session.send_command(cmd_name, params)
            result = await future

            if result_fn is not None and not raw_result:
                # Convert the result if we have a conversion function defined
                if asyncio.iscoroutinefunction(result_fn):
                    result = await result_fn(self, result)
                else:
                    result = result_fn(self, result)
            return result

        # Overwrite the method on the owner class with the wrapper
        setattr(owner, name, inner)


class BidiModule:

    def __init__(self, session: "BidiSession"):
        self.session = session


def to_camelcase(name: str) -> str:
    """Convert a python style method name foo_bar to a BiDi command name fooBar"""
    parts = name.split("_")
    parts[0] = parts[0].lower()
    for i in range(1, len(parts)):
        parts[i] = parts[i].title()
    return "".join(parts)


def remove_undefined(obj: Mapping[str, Any]) -> Mapping[str, Any]:
    return {key: value for key, value in obj.items() if value != UNDEFINED}
