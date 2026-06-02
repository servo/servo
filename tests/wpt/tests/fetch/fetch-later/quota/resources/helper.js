'use strict';

// See the max returned value from
// https://whatpr.org/fetch/1647.html#available-deferred-fetch-quota
const QUOTA_PER_ORIGIN = 64 * 1024;  // 64 kibibytes.
// See the "minimal quota" from
// https://whatpr.org/fetch/1647.html#available-deferred-fetch-quota
const QUOTA_PER_CROSS_ORIGIN = 8 * 1024;  // 8 kibibytes.
// See the "quotaForRequestOrigin" from
// https://whatpr.org/fetch/1647.html#available-deferred-fetch-quota
const QUOTA_PER_REPORTING_ORIGIN = 64 * 1024;  // 64 kibibytes.


/**
 * Calculates max possible remaining quota for a fetchLater request.
 *
 * Note that this function does NOT know how many fetchLater calls has been made
 * before calling, nor does it know anything about requests within the URL
 * origin, and both of them take up quota for the origin.
 *
 * @param {number} maxQuota The max quota to deduct from. Caller must ensure
 *                          this is actually used by its owned context.
 * @param {String} url The URL for a fetchLater call.
 * @param {Headers=} headers The headers for a fetchLater call.
 * @return {number} The max possible per-origin quota for fetchLater calls.
 */
function getRemainingQuota(maxQuota, url, headers = {}) {
  let quota = maxQuota - url.length;
  if (headers instanceof Headers) {
    for (const kv of headers.entries()) {
      quota -= kv[0].length + kv[1].length;
    }
  }
  return quota > 0 ? quota : 0;
}
