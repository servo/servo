# These functions are used by FLEDGE to determine the logic for the ad seller.
# For our testing purposes, we only need the minimal amount of boilerplate
# code in place to allow them to be invoked properly and move the FLEDGE
# process along. The tests do not deal with reporting results, so we leave
# `reportResult` empty. See `generateURNFromFledge` in "utils.js" to see how
# this file is used.

def main(request, response):
  # Set up response headers.
  headers = [
    ('Content-Type', 'Application/Javascript'),
    ('X-Allow-FLEDGE', 'true')
  ]

  # TODO: Insert any code here that should be mutated based on URL parameters.
  # Keep for now as a placeholder/example.
  score_ad_content = ''

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
        {score_ad_content}
        return 2*bid;
      }}
    '''
  )

  report_result = (
    '''function reportResult(
      auctionConfig,
      browserSignals) {
        return;
      }
    '''
  )

  content = f'{score_ad}\n{report_result}'

  return (headers, content)
