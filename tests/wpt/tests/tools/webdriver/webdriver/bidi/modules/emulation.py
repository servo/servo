from typing import Any, Dict, List, Literal, Mapping

from ._module import BidiModule, command
from ..undefined import UNDEFINED, Maybe, Nullable


class CoordinatesOptions(Dict[str, Any]):
    def __init__(
            self,
            latitude: float,
            longitude: float,
            accuracy: Maybe[float] = UNDEFINED,
            altitude: Maybe[Nullable[float]] = UNDEFINED,
            altitude_accuracy: Maybe[Nullable[float]] = UNDEFINED,
            heading: Maybe[Nullable[float]] = UNDEFINED,
            speed: Maybe[Nullable[float]] = UNDEFINED,
    ):
        self["latitude"] = latitude
        self["longitude"] = longitude
        self["accuracy"] = accuracy
        self["altitude"] = altitude
        self["altitudeAccuracy"] = altitude_accuracy
        self["heading"] = heading
        self["speed"] = speed


class Emulation(BidiModule):
    @command
    def set_geolocation_override(
            self,
            coordinates: Maybe[Nullable[CoordinatesOptions]] = UNDEFINED,
            error: Maybe[Dict[str, Any]] = UNDEFINED,
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "coordinates": coordinates,
            "error": error,
            "contexts": contexts,
            "userContexts": user_contexts
        }

    @command
    def set_locale_override(
            self,
            locale: Nullable[str],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "locale": locale,
            "contexts": contexts,
            "userContexts": user_contexts
        }

    @command
    def set_scripting_enabled(
            self,
            enabled: Nullable[Literal[False]],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "enabled": enabled,
            "contexts": contexts,
            "userContexts": user_contexts,
        }

    @command
    def set_screen_orientation_override(
            self,
            screen_orientation: Nullable[Dict[str, Any]],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "screenOrientation": screen_orientation,
            "contexts": contexts,
            "userContexts": user_contexts
        }

    @command
    def set_timezone_override(
            self,
            timezone: Nullable[str],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "timezone": timezone,
            "contexts": contexts,
            "userContexts": user_contexts
        }

    @command
    def set_user_agent_override(
            self,
            user_agent: Nullable[str],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "userAgent": user_agent,
            "contexts": contexts,
            "userContexts": user_contexts,
        }

    @command
    def set_network_conditions(
            self,
            network_conditions: Nullable[Dict[str, Any]],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "networkConditions": network_conditions,
            "contexts": contexts,
            "userContexts": user_contexts,
        }
