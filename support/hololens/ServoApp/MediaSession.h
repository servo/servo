#pragma once

namespace winrt::servo {
	enum PlaybackState {
		NONE = 1,
		PLAYING,
		PAUSED
	};

	enum MediaSessionAction {
		PLAY = 1,
		PAUSE,
		SEEK_BACKWARD,
		SEEK_FORWARD,
		PREVIOUS_TRACK,
		NEXT_TRACK,
		SKIP_AD,
		STOP,
		SEEK_TO
	};
}