Hyper-h2 is written and maintained by Cory Benfield and various contributors:

Development Lead
````````````````

- Cory Benfield <cory@lukasa.co.uk>

Contributors
````````````

In chronological order:

- Robert Collins (@rbtcollins)

  - Provided invaluable and substantial early input into API design and layout.
  - Added code preventing ``Proxy-Authorization`` from getting added to HPACK
    compression contexts.

- Maximilian Hils (@maximilianhils)

  - Added asyncio example.

- Alex Chan (@alexwlchan)

  - Fixed docstring, added URLs to README.

- Glyph Lefkowitz (@glyph)

  - Improved example Twisted server.

- Thomas Kriechbaumer (@Kriechi)

  - Fixed incorrect arguments being passed to ``StreamIDTooLowError``.
  - Added new arguments to ``close_connection``.

- WeiZheng Xu (@boyxuper)

  - Reported a bug relating to hyper-h2's updating of the connection window in
    response to SETTINGS_INITIAL_WINDOW_SIZE.

- Evgeny Tataurov (@etataurov)

  - Added the ``additional_data`` field to the ``ConnectionTerminated`` event.

- Brett Cannon (@brettcannon)

  - Changed Travis status icon to SVG.
  - Documentation improvements.

- Felix Yan (@felixonmars)

  - Widened allowed version numbers of enum34.
  - Updated test requirements.

- Keith Dart (@kdart)

  - Fixed curio example server flow control problems.

- Gil Gonçalves (@LuRsT)

  - Added code forbidding non-RFC 7540 pseudo-headers.

- Louis Taylor (@kragniz)

  - Cleaned up the README

- Berker Peksag (@berkerpeksag)

  - Improved the docstring for ``StreamIDTooLowError``.

- Adrian Lewis (@aidylewis)

  - Fixed the broken Twisted HEAD request example.
  - Added verification logic for ensuring that responses to HEAD requests have
    no body.

- Lorenzo (@Mec-iS)

  - Changed documentation to stop using dictionaries for header blocks.

- Kracekumar Ramaraj (@kracekumar)

  - Cleaned up Twisted example.

- @mlvnd

  - Cleaned up curio example.

- Tom Offermann (@toffer)

  - Added Tornado example.

- Tarashish Mishra (@sunu)

  - Added code to reject header fields with leading/trailing whitespace.
  - Added code to remove leading/trailing whitespace from sent header fields.

- Nate Prewitt (@nateprewitt)

  - Added code to validate that trailers do not contain pseudo-header fields.

- Chun-Han, Hsiao (@chhsiao90)

  - Fixed a bug with invalid ``HTTP2-Settings`` header output in plaintext
    upgrade.

- Bhavishya (@bhavishyagopesh)

  - Added support for equality testing to ``h2.settings.Settings`` objects.

- Fred Thomsen (@fredthomsen)

  - Added logging.
  - Enhance equality testing of ``h2.settings.Settings`` objects with
    ``hypothesis``.
