from collections import defaultdict

from typing import (Any,
                    ClassVar,
                    List,
                    Mapping,
                    MutableMapping,
                    Optional,
                    Sequence,
                    Set,
                    Type,
                    TypeVar,
                    Union)

from ._module import BidiModule, command

InputSourceType = TypeVar('InputSourceType', bound="InputSource")


class Action:
    action_type: ClassVar[str]

    def to_json(self) -> MutableMapping[str, Any]:
        return {"type": self.action_type}


class PauseAction(Action):
    action_type = "pause"

    def __init__(self, duration: Optional[int] = None):
        self.duration = duration

    def to_json(self) -> MutableMapping[str, Any]:
        rv = super().to_json()
        if self.duration is not None:
            rv["duration"] = self.duration
        return rv


class KeyAction(Action):

    def __init__(self, key: str):
        self.value = key

    def to_json(self) -> MutableMapping[str, Any]:
        rv = super().to_json()
        rv["value"] = self.value
        return rv


class KeyUpAction(KeyAction):
    action_type = "keyUp"


class KeyDownAction(KeyAction):
    action_type = "keyDown"


class PointerAction(Action):

    def __init__(self,
                 button: Optional[int] = None,
                 x: Optional[int] = None,
                 y: Optional[int] = None,
                 duration: Optional[int] = None,
                 origin: Optional[Union[str, Mapping[str, Any]]] = None,
                 width: Optional[int] = None,
                 height: Optional[int] = None,
                 pressure: Optional[float] = None,
                 tangential_pressure: Optional[float] = None,
                 tilt_x: Optional[int] = None,
                 tilt_y: Optional[int] = None,
                 twist: Optional[int] = None,
                 altitude_angle: Optional[float] = None,
                 azimuth_angle: Optional[float] = None):
        self.button = button
        self.x = x
        self.y = y
        self.duration = duration
        self.origin = origin
        self.width = width
        self.height = height
        self.pressure = pressure
        self.tangential_pressure = tangential_pressure
        self.tilt_x = tilt_x
        self.tilt_y = tilt_y
        self.twist = twist
        self.altitude_angle = altitude_angle
        self.azimuth_angle = azimuth_angle

    def to_json(self) -> MutableMapping[str, Any]:
        rv = super().to_json()
        if self.button is not None:
            rv["button"] = self.button
        if self.x is not None:
            rv["x"] = self.x
        if self.y is not None:
            rv["y"] = self.y
        if self.duration is not None:
            rv["duration"] = self.duration
        if self.origin is not None:
            rv["origin"] = self.origin
        if self.width is not None:
            rv["width"] = self.width
        if self.height is not None:
            rv["height"] = self.height
        if self.pressure is not None:
            rv["pressure"] = self.pressure
        if self.tangential_pressure is not None:
            rv["tangentialPressure"] = self.tangential_pressure
        if self.tilt_x is not None:
            rv["tiltX"] = self.tilt_x
        if self.tilt_y is not None:
            rv["tiltY"] = self.tilt_y
        if self.twist is not None:
            rv["twist"] = self.twist
        if self.altitude_angle is not None:
            rv["altitudeAngle"] = self.altitude_angle
        if self.azimuth_angle is not None:
            rv["azimuthAngle"] = self.azimuth_angle
        return rv


class PointerDownAction(PointerAction):
    action_type = "pointerDown"

    def __init__(self,
                 button: int,
                 width: Optional[int] = None,
                 height: Optional[int] = None,
                 pressure: Optional[float] = None,
                 tangential_pressure: Optional[float] = None,
                 tilt_x: Optional[int] = None,
                 tilt_y: Optional[int] = None,
                 twist: Optional[int] = None,
                 altitude_angle: Optional[float] = None,
                 azimuth_angle: Optional[float] = None):
        super().__init__(button=button,
                         x=None,
                         y=None,
                         duration=None,
                         origin=None,
                         width=width,
                         height=height,
                         pressure=pressure,
                         tangential_pressure=tangential_pressure,
                         tilt_x=tilt_x,
                         tilt_y=tilt_y,
                         twist=twist,
                         altitude_angle=altitude_angle,
                         azimuth_angle=azimuth_angle)


class PointerUpAction(PointerAction):
    action_type = "pointerUp"

    def __init__(self,
                 button: int,
                 width: Optional[int] = None,
                 height: Optional[int] = None,
                 pressure: Optional[float] = None,
                 tangential_pressure: Optional[float] = None,
                 tilt_x: Optional[int] = None,
                 tilt_y: Optional[int] = None,
                 twist: Optional[int] = None,
                 altitude_angle: Optional[float] = None,
                 azimuth_angle: Optional[float] = None):
        super().__init__(button=button,
                         x=None,
                         y=None,
                         duration=None,
                         origin=None,
                         width=width,
                         height=height,
                         pressure=pressure,
                         tangential_pressure=tangential_pressure,
                         tilt_x=tilt_x,
                         tilt_y=tilt_y,
                         twist=twist,
                         altitude_angle=altitude_angle,
                         azimuth_angle=azimuth_angle)


class PointerMoveAction(PointerAction):
    action_type = "pointerMove"

    def __init__(self,
                 x: int,
                 y: int,
                 duration: Optional[int] = None,
                 origin: Optional[Union[str, Mapping[str, Any]]] = None,
                 width: Optional[int] = None,
                 height: Optional[int] = None,
                 pressure: Optional[float] = None,
                 tangential_pressure: Optional[float] = None,
                 tilt_x: Optional[int] = None,
                 tilt_y: Optional[int] = None,
                 twist: Optional[int] = None,
                 altitude_angle: Optional[float] = None,
                 azimuth_angle: Optional[float] = None):
        super().__init__(button=None,
                         x=x,
                         y=y,
                         duration=duration,
                         origin=origin,
                         width=width,
                         height=height,
                         pressure=pressure,
                         tangential_pressure=tangential_pressure,
                         tilt_x=tilt_x,
                         tilt_y=tilt_y,
                         twist=twist,
                         altitude_angle=altitude_angle,
                         azimuth_angle=azimuth_angle)


class WheelScrollAction(Action):
    action_type = "scroll"

    def __init__(self,
                 x: int,
                 y: int,
                 delta_x: int,
                 delta_y: int,
                 duration: Optional[int] = None,
                 origin: Optional[Union[str, Mapping[str, Any]]] = None):
        self.x = x
        self.y = y
        self.delta_x = delta_x
        self.delta_y = delta_y
        self.duration = duration
        self.origin = origin

    def to_json(self) -> MutableMapping[str, Any]:
        rv = super().to_json()
        rv.update({
            "x": self.x,
            "y": self.y,
            "deltaX": self.delta_x,
            "deltaY": self.delta_y
        })
        if self.duration is not None:
            rv["duration"] = self.duration
        if self.origin is not None:
            rv["origin"] = self.origin
        return rv


class InputSource:
    input_type: ClassVar[str]

    def __init__(self, input_id: str, **kwargs: Any):
        """Represents a sequence of actions of one type for one input source.

        :param input_id: ID of input source.
        """
        self.id = input_id
        self.actions: List[Action] = []

    def __len__(self) -> int:
        return len(self.actions)

    def to_json(self, total_ticks: int) -> MutableMapping[str, Any]:
        actions = [item.to_json() for item in self.actions]
        for i in range(total_ticks - len(self)):
            actions.append(PauseAction().to_json())

        return {"id": self.id, "type": self.input_type, "actions": actions}

    def done(self) -> List[Action]:
        return self.actions

    def pause(self: InputSourceType,
              duration: Optional[int] = None) -> InputSourceType:
        self.actions.append(PauseAction(duration))
        return self


class KeyInputSource(InputSource):
    input_type = "key"

    def key_down(self, key: str) -> "KeyInputSource":
        self.actions.append(KeyDownAction(key))
        return self

    def key_up(self, key: str) -> "KeyInputSource":
        self.actions.append(KeyUpAction(key))
        return self

    def send_keys(self, keys: str) -> "KeyInputSource":
        for c in keys:
            self.key_down(c)
            self.key_up(c)
        return self


class PointerInputSource(InputSource):
    input_type = "pointer"

    def __init__(self, input_id: str, pointer_type: str = "mouse"):
        super().__init__(input_id)
        self.parameters = {"pointerType": pointer_type}

    def to_json(self, total_ticks: int) -> MutableMapping[str, Any]:
        rv = super().to_json(total_ticks)
        rv["parameters"] = self.parameters
        return rv

    def pointer_down(self, button: int, **kwargs: Any) -> "PointerInputSource":
        self.actions.append(PointerDownAction(button, **kwargs))
        return self

    def pointer_up(self, button: int, **kwargs: Any) -> "PointerInputSource":
        self.actions.append(PointerUpAction(button, **kwargs))
        return self

    def pointer_move(self,
                     x: int,
                     y: int,
                     duration: Optional[int] = None,
                     origin: Union[str, Mapping[str, Any]] = "viewport",
                     **kwargs: Any) -> "PointerInputSource":
        self.actions.append(
            PointerMoveAction(x, y, duration=duration, origin=origin,
                              **kwargs))
        return self


class WheelInputSource(InputSource):
    input_type = "wheel"

    def scroll(self,
               x: int,
               y: int,
               delta_x: int = 0,
               delta_y: int = 0,
               duration: Optional[int] = None,
               origin: Union[str, Mapping[str, Any]] = "viewport") -> "WheelInputSource":
        self.actions.append(WheelScrollAction(x,
                                              y,
                                              delta_x=delta_x,
                                              delta_y=delta_y,
                                              duration=duration,
                                              origin=origin))
        return self


class Actions:

    def __init__(self) -> None:
        self.input_sources: List[InputSource] = []
        self.seen_names: MutableMapping[str, Set[str]] = defaultdict(set)

    def _add_source(self,
                    cls: Type[InputSourceType],
                    input_id: Optional[str] = None,
                    **kwargs: Any) -> InputSourceType:
        input_type = cls.input_type
        if input_id is None:
            i = 0
            input_id = f"{input_type}-{i}"
            while input_id in self.seen_names[input_type]:
                i += 1
                input_id = f"{input_type}-{i}"
        else:
            if input_id in self.seen_names[input_type]:
                raise ValueError(f"Duplicate input id ${input_id}")

        self.seen_names[input_type].add(input_id)
        rv = cls(input_id, **kwargs)
        self.input_sources.append(rv)
        return rv

    def add_key(self, input_id: Optional[str] = None) -> "KeyInputSource":
        return self._add_source(KeyInputSource, input_id)

    def add_pointer(self,
                    input_id: Optional[str] = None,
                    pointer_type: str = "mouse") -> "PointerInputSource":
        return self._add_source(PointerInputSource,
                                input_id,
                                pointer_type=pointer_type)

    def add_wheel(self, input_id: Optional[str] = None) -> "WheelInputSource":
        return self._add_source(WheelInputSource, input_id)

    def to_json(self) -> Sequence[Mapping[str, Any]]:
        num_ticks = max(len(item) for item in self.input_sources)
        return [item.to_json(num_ticks) for item in self.input_sources]


class Input(BidiModule):

    @command
    def perform_actions(self,
                        actions: Union[Actions, List[Any]],
                        context: str
                        ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "context": context
        }
        if isinstance(actions, Actions):
            params["actions"] = actions.to_json()
        else:
            params["actions"] = actions
        return params

    @command
    def release_actions(self, context: str) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"context": context}
        return params


def get_element_origin(element: Any) -> Mapping[str, Any]:
    return {"type": "element", "element": {"sharedId": element["sharedId"]}}
