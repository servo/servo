from typing import Any, Dict, List, Mapping, MutableMapping, Optional, Union

from ._module import BidiModule, command
from ..undefined import UNDEFINED, Undefined


class CoordinatesOptions(Dict[str, Any]):
    def __init__(
        self,
        latitude: float,
        longitude: float,
        accuracy: Optional[float] = None,
        altitude: Optional[float] = None,
        altitude_accuracy: Optional[float] = None,
        heading: Optional[float] = None,
        speed: Optional[float] = None,
    ):
        self["latitude"] = latitude
        self["longitude"] = longitude

        if accuracy is not None:
            self["accuracy"] = accuracy
        if altitude is not None:
            self["altitude"] = altitude
        if altitude_accuracy is not None:
            self["altitudeAccuracy"] = altitude_accuracy
        if heading is not None:
            self["heading"] = heading
        if speed is not None:
            self["speed"] = speed


class Emulation(BidiModule):
    @command
    def set_geolocation_override(
        self,
        coordinates: Union[CoordinatesOptions, Undefined] = UNDEFINED,
        error: Optional[Dict[str, Any]] = None,
        contexts: Optional[List[str]] = None,
        user_contexts: Optional[List[str]] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if coordinates is not UNDEFINED:
            params["coordinates"] = coordinates
        if error is not None:
            params["error"] = error
        if contexts is not None:
            params["contexts"] = contexts
        if user_contexts is not None:
            params["userContexts"] = user_contexts

        return params

    @command
    def set_locale_override(
        self,
        locale: Union[str, None],
        contexts: Optional[List[str]] = None,
        user_contexts: Optional[List[str]] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "locale": locale
        }

        if contexts is not None:
            params["contexts"] = contexts
        if user_contexts is not None:
            params["userContexts"] = user_contexts

        return params

    @command
    def set_screen_orientation_override(
        self,
        screen_orientation:Dict[str, Any],
        contexts: Optional[List[str]] = None,
        user_contexts: Optional[List[str]] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "screenOrientation": screen_orientation
        }

        if contexts is not None:
            params["contexts"] = contexts
        if user_contexts is not None:
            params["userContexts"] = user_contexts

        return params

    @command
    def set_timezone_override(
            self,
            timezone: Union[str, None],
            contexts: Optional[List[str]] = None,
            user_contexts: Optional[List[str]] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "timezone": timezone
        }

        if contexts is not None:
            params["contexts"] = contexts
        if user_contexts is not None:
            params["userContexts"] = user_contexts

        return params
