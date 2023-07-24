/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/
/**
 * Equivalent of `setTimeout`, but redirects to WPT's `step_timeout` when it is defined.
 */
export const timeout = typeof step_timeout !== 'undefined' ? step_timeout : setTimeout;
