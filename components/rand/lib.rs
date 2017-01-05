/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// A random number generator which shares one instance of an `OsRng`.
///
/// A problem with `OsRng`, which is inherited by `StdRng` and so
/// `ThreadRng`, is that it reads from `/dev/random`, and so consumes
/// a file descriptor. For multi-threaded applications like Servo,
/// it is easy to exhaust the supply of file descriptors this way.
///
/// This crate fixes that, by only using one `OsRng`, which is just
/// used to seed and re-seed an `ServoRng`.

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rand;

pub use rand::{Rand, Rng, SeedableRng};
#[cfg(target_pointer_width = "64")]
use rand::isaac::Isaac64Rng as IsaacWordRng;
#[cfg(target_pointer_width = "32")]
use rand::isaac::IsaacRng as IsaacWordRng;
use rand::os::OsRng;
use rand::reseeding::{ReseedingRng, Reseeder};
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::sync::Mutex;
use std::u64;

// Slightly annoying having to cast between sizes.

#[cfg(target_pointer_width = "64")]
fn as_isaac_seed(seed: &[usize]) -> &[u64] {
    unsafe { mem::transmute(seed) }
}

#[cfg(target_pointer_width = "32")]
fn as_isaac_seed(seed: &[usize]) -> &[u32] {
    unsafe { mem::transmute(seed) }
}

// The shared RNG which may hold on to a file descriptor
lazy_static! {
    static ref OS_RNG: Mutex<OsRng> = match OsRng::new() {
        Ok(r) => Mutex::new(r),
        Err(e) => panic!("Failed to seed OsRng: {}", e),
    };
}

// Generate 32K of data between reseedings
const RESEED_THRESHOLD: u64 = 32_768;

// An in-memory RNG that only uses the shared file descriptor for seeding and reseeding.
pub struct ServoRng {
    rng: ReseedingRng<IsaacWordRng, ServoReseeder>,
}

impl Rng for ServoRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }
}

impl<'a> SeedableRng<&'a [usize]> for ServoRng {
    /// Create a manually-reseeding instane of `ServoRng`.
    ///
    /// Note that this RNG does not reseed itself, so care is needed to reseed the RNG
    /// is required to be cryptographically sound.
    fn from_seed(seed: &[usize]) -> ServoRng {
        debug!("Creating new manually-reseeded ServoRng.");
        let isaac_rng = IsaacWordRng::from_seed(as_isaac_seed(seed));
        let reseeding_rng = ReseedingRng::new(isaac_rng, u64::MAX, ServoReseeder);
        ServoRng { rng: reseeding_rng }
    }
    /// Reseed the RNG.
    fn reseed(&mut self, seed: &'a [usize]) {
        debug!("Manually reseeding ServoRng.");
        self.rng.reseed((ServoReseeder, as_isaac_seed(seed)))
    }
}

impl ServoRng {
    /// Create an auto-reseeding instance of `ServoRng`.
    ///
    /// This uses the shared `OsRng`, so avoids consuming
    /// a file descriptor.
    pub fn new() -> ServoRng {
        debug!("Creating new ServoRng.");
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        let isaac_rng = IsaacWordRng::rand(&mut *os_rng);
        let reseeding_rng = ReseedingRng::new(isaac_rng, RESEED_THRESHOLD, ServoReseeder);
        ServoRng { rng: reseeding_rng }
    }
}

// The reseeder for the in-memory RNG.
struct ServoReseeder;

impl Reseeder<IsaacWordRng> for ServoReseeder {
    fn reseed(&mut self, rng: &mut IsaacWordRng) {
        debug!("Reseeding ServoRng.");
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        *rng = IsaacWordRng::rand(&mut *os_rng);
    }
}

impl Default for ServoReseeder {
    fn default() -> ServoReseeder {
        ServoReseeder
    }
}

// A thread-local RNG, designed as a drop-in replacement for rand::ThreadRng.
#[derive(Clone)]
pub struct ServoThreadRng {
    rng: Rc<RefCell<ServoRng>>,
}

// A thread-local RNG, designed as a drop-in replacement for rand::thread_rng.
pub fn thread_rng() -> ServoThreadRng {
    SERVO_THREAD_RNG.with(|t| t.clone())
}

thread_local! {
    static SERVO_THREAD_RNG: ServoThreadRng = ServoThreadRng { rng: Rc::new(RefCell::new(ServoRng::new())) };
}

impl Rng for ServoThreadRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.borrow_mut().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.borrow_mut().next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        self.rng.borrow_mut().fill_bytes(bytes)
    }
}

// Generates a random value using the thread-local random number generator.
// A drop-in replacement for rand::random.
#[inline]
pub fn random<T: Rand>() -> T {
    thread_rng().gen()
}
