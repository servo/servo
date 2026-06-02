# These functions are used by FLEDGE to determine the logic for the ad seller.
# For our testing purposes, we only need the minimal amount of boilerplate
# code in place to allow them to be invoked properly and move the FLEDGE
# process along. The tests do not deal with reporting results, so we leave
# `reportResult` empty. See `generateURNFromFledge` in "utils.js" to see how
# this file is used.

from wptserve.utils import isomorphic_decode

def main(request, response):
  # Set up response headers.
  headers = [
    ('Content-Type', 'Application/Javascript'),
    ('Ad-Auction-Allowed', 'true')
  ]

  # Parse URL params.
  requested_size = request.GET.first(b"requested-size", None)

  # Use URL params to modify Javascript.
  requested_size_check = ''
  if requested_size is not None:
    # request.GET stores URL keys and values in iso-8859-1 binary encoding. We
    # have to decode the values back to a string to parse width/height. Don't
    # bother sanitizing the size, because it is sanitized before auction logic
    # runs already.
    width, height = isomorphic_decode(requested_size).split('-')

    requested_size_check = (
      f'''
        if (!(auctionConfig.requestedSize.width === '{width}') &&
             (auctionConfig.requestedSize.height === '{height}')) {{
          throw new Error('requestedSize missing/incorrect in auctionConfig');
        }}
      '''
    )

  # Generate Javascript.
  # Note: Python fstrings use double-brackets ( {{, }} ) to insert bracket
  # literals instead of substitution sequences.
  score_ad = (
    f'''function scoreAd(
      adMetadata,
      bid,
      auctionConfig,
      trustedScoringSignals,
      browserSignals) {{
        {requested_size_check}
        return 2*bid;
      }}
    '''
  )

  report_result = (
    f'''function reportResult(
      auctionConfig,
      browserSignals) {{
        {requested_size_check}
        return;
      }}
    '''
  )

  content = f'{score_ad}\n{report_result}'

  return (headers, content)
