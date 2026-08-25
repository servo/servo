/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../common/util/util.js';import { kValue } from './constants.js';

/**
 * Seed-able deterministic pseudo random generator for the WebGPU CTS
 *
 * This generator requires setting a seed value and the sequence of values
 * generated is deterministic based on the seed.
 *
 * This generator is intended to be a replacement for Math.random().
 *
 * This generator is not cryptographically secure, though nothing in the CTS
 * should be needing cryptographic security.
 *
 * The current implementation is based on TinyMT
 * (https://github.com/MersenneTwister-Lab/TinyMT), which is a version of
 * Mersenne Twister that has reduced the internal state size at the cost of
 * shortening the period length of the generated sequence. The period is still
 * 2^127 - 1 entries long, so should be sufficient for use in the CTS, but it is
 * less costly to create multiple instances of the class.
 */
export class PRNG {
  // Storing variables for temper() as members, so they don't need to be
  // reallocated per call to temper()


  // Storing variables for next() as members, so they don't need to be
  // reallocated per call to next()


  // Generator internal state


  // Default tuning parameters for TinyMT.
  // These are tested to not generate an all zero initial state.
  static kMat1 = 0x8f7011ee;
  static kMat2 = 0xfc78ff1f;
  static kTMat = 0x3793fdff;

  // TinyMT algorithm internal magic numbers
  static kMask = 0x7fffffff;
  static kMinLoop = 8;
  static kPreLoop = 8;
  static kSH0 = 1;
  static kSH1 = 10;
  static kSH8 = 8;

  // u32.max + 1, used to scale the u32 value from temper() to [0, 1).
  static kRandomDivisor = 4294967296.0;

  /**
   * constructor
   *
   * @param seed value used to initialize random number sequence. Results are
   *             guaranteed to be deterministic based on this.
   *             This value must be in the range of unsigned 32-bit integers.
   *             Non-integers will be rounded.
   */
  constructor(seed) {
    assert(seed >= 0 && seed <= kValue.u32.max, 'seed to PRNG needs to a u32');

    this.t_vars = new Uint32Array(2);
    this.n_vars = new Uint32Array(2);

    this.state = new Uint32Array([Math.round(seed), PRNG.kMat1, PRNG.kMat2, PRNG.kTMat]);
    for (let i = 1; i < PRNG.kMinLoop; i++) {
      this.state[i & 3] ^=
      i + Math.imul(1812433253, this.state[i - 1 & 3] ^ this.state[i - 1 & 3] >>> 30);
    }

    // Check that the initial state isn't all 0s, since the algorithm assumes
    // that this never occurs
    assert(
      (this.state[0] & PRNG.kMask) !== 0 ||
      this.state[1] !== 0 ||
      this.state[2] !== 0 ||
      this.state[2] !== 0,
      'Initialization of PRNG unexpectedly generated all 0s initial state, this means the tuning parameters are bad'
    );

    for (let i = 0; i < PRNG.kPreLoop; i++) {
      this.next();
    }
  }

  /** Advances the internal state to the next values */
  next() {
    this.n_vars[0] = this.state[0] & PRNG.kMask ^ this.state[1] ^ this.state[2];
    this.n_vars[1] = this.state[3];
    this.n_vars[0] ^= this.n_vars[0] << PRNG.kSH0;
    this.n_vars[1] ^= this.n_vars[1] >>> PRNG.kSH0 ^ this.n_vars[0];
    this.state[0] = this.state[1];
    this.state[1] = this.state[2];
    this.state[2] = this.n_vars[0] ^ this.n_vars[1] << PRNG.kSH1;
    this.state[3] = this.n_vars[1];
    if ((this.n_vars[1] & 1) !== 0) {
      this.state[1] ^= PRNG.kMat1;
      this.state[2] ^= PRNG.kMat2;
    }
  }

  /** @returns a 32-bit unsigned integer based on the current state */
  temper() {
    this.t_vars[0] = this.state[3];
    this.t_vars[1] = this.state[0] + (this.state[2] >>> PRNG.kSH8);
    this.t_vars[0] ^= this.t_vars[1];
    if ((this.t_vars[1] & 1) !== 0) {
      this.t_vars[0] ^= PRNG.kTMat;
    }
    return this.t_vars[0];
  }

  /** @returns a value on the range of [0, 1)  and advances the state */
  random() {
    this.next();
    return this.temper() / PRNG.kRandomDivisor;
  }

  /** @returns a 32-bit unsigned integer value and advances the state */
  randomU32() {
    this.next();
    return this.temper();
  }

  /** @returns a uniformly selected integer in [0, N-1].  N must be at least 1 and at most 2**32. */
  uniformInt(N) {
    const upperBound = (1 << 16) * (1 << 16);
    assert(N === Math.trunc(N)); // It's an integer
    assert(N > 0, `${N} must be positive`);
    assert(N <= upperBound, `${N} is too big, should be at most ${upperBound}`);

    // Use a method described in The Stanford GraphBase,
    // Donald E. Knuth (New York: ACM Press, 1994), viii+576pp.
    // Co-published by Addison-Wesley Publishing Company.
    // See GB_FLIP, section 12, Uniform Integers.

    // A naive algorithm would take a random u32 value X, and then
    // return (X % N).  But this is biased toward smaller values
    // when N is not a power of 2.  As Knuth writes, if N is
    // (2**32) / 3, this naive algorithm would return values
    // less than N/2 about 2/3 of the time.

    // Instead, we eliminate the bias by discarding samples when
    // they would have been biased. The "keep zone", so to speak,
    // must be a multiple of N.  We make the algorithm efficient
    // by maximizing the size of the keep zone: Find the largest
    // multiple of N that fits within a u32.
    // On average, this algorithm will discard 2 or fewer samples.

    // Find the largest multiple of N that fits below upperBound.
    const keepZoneSize = upperBound - upperBound % N;
    assert(keepZoneSize % N === 0);
    // It covers a big chunk of the whole u32 range.
    assert(keepZoneSize >= upperBound / 2);
    // Draw u32 values until we find one in the keep zone.
    let candidate;
    do {
      candidate = this.randomU32();
    } while (candidate >= keepZoneSize);
    // Now return the candidate, but folding away multiples of N.
    return candidate % N;
  }
}