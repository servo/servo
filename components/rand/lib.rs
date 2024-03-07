/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use std::u64;

use lazy_static::lazy_static;
use log::trace;
/// A random number generator which shares one instance of an `OsRng`.
///
/// A problem with `OsRng`, which is inherited by `StdRng` and so
/// `ThreadRng`, is that it reads from `/dev/random`, and so consumes
/// a file descriptor. For multi-threaded applications like Servo,
/// it is easy to exhaust the supply of file descriptors this way.
///
/// This crate fixes that, by only using one `OsRng`, which is just
/// used to seed and re-seed an `ServoRng`.
use rand::distributions::{Distribution, Standard};
use rand::rngs::adapter::ReseedingRng;
use rand::rngs::OsRng;
pub use rand::seq::SliceRandom;
pub use rand::{Rng, RngCore, SeedableRng};
use rand_isaac::isaac::IsaacCore;
use uuid::{Builder, Uuid};

// The shared RNG which may hold on to a file descriptor
lazy_static! {
    static ref OS_RNG: Mutex<OsRng> = Mutex::new(OsRng);
}

// Generate 32K of data between reseedings
const RESEED_THRESHOLD: u64 = 32_768;

// An in-memory RNG that only uses the shared file descriptor for seeding and reseeding.
pub struct ServoRng {
    rng: ReseedingRng<IsaacCore, ServoReseeder>,
}

impl RngCore for ServoRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        self.rng.fill_bytes(bytes)
    }

    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> std::result::Result<(), rand_core::Error> {
        self.rng.try_fill_bytes(bytes)
    }
}

#[derive(Default)]
pub struct Seed([u8; 32]);

impl AsMut<[u8]> for Seed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl SeedableRng for ServoRng {
    type Seed = Seed;

    // This function is used in the reseeding process of rand hence why the RESEED_THRESHOLD is
    // used.
    fn from_seed(seed: Seed) -> ServoRng {
        trace!("Creating a new ServoRng.");
        let isaac_rng = IsaacCore::from_seed(seed.0);
        let reseeding_rng = ReseedingRng::new(isaac_rng, RESEED_THRESHOLD, ServoReseeder);
        ServoRng { rng: reseeding_rng }
    }
}

impl ServoRng {
    /// Create a manually-reseeding instance of `ServoRng`.
    ///
    /// Note that this RNG does not reseed itself, so care is needed to reseed the RNG
    /// is required to be cryptographically sound.
    pub fn new_manually_reseeded(seed: u64) -> ServoRng {
        trace!("Creating a new manually-reseeded ServoRng.");
        let isaac_rng = IsaacCore::seed_from_u64(seed);
        let reseeding_rng = ReseedingRng::new(isaac_rng, u64::MAX, ServoReseeder);
        ServoRng { rng: reseeding_rng }
    }
}

impl Default for ServoRng {
    /// Create an auto-reseeding instance of `ServoRng`.
    ///
    /// This uses the shared `OsRng`, so avoids consuming
    /// a file descriptor.
    fn default() -> Self {
        trace!("Creating new ServoRng.");
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        let isaac_rng = IsaacCore::from_rng(&mut *os_rng).unwrap();
        let reseeding_rng = ReseedingRng::new(isaac_rng, RESEED_THRESHOLD, ServoReseeder);
        ServoRng { rng: reseeding_rng }
    }
}

// The reseeder for the in-memory RNG.
struct ServoReseeder;

impl RngCore for ServoReseeder {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        os_rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        os_rng.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        os_rng.fill_bytes(bytes)
    }

    #[inline]
    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> std::result::Result<(), rand_core::Error> {
        let mut os_rng = OS_RNG.lock().expect("Poisoned lock.");
        os_rng.try_fill_bytes(bytes)
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
    static SERVO_THREAD_RNG: ServoThreadRng = ServoThreadRng { rng: Rc::new(RefCell::new(ServoRng::default())) };
}

impl RngCore for ServoThreadRng {
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

    #[inline]
    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> std::result::Result<(), rand_core::Error> {
        (self.rng.borrow_mut()).try_fill_bytes(bytes)
    }
}

// Generates a random value using the thread-local random number generator.
// A drop-in replacement for rand::random.
#[inline]
pub fn random<T>() -> T
where
    Standard: Distribution<T>,
{
    thread_rng().gen()
}

// TODO(eijebong): Replace calls to this by random once `uuid::Uuid` implements `rand::Rand` again.
#[inline]
pub fn random_uuid() -> Uuid {
    let mut bytes = [0; 16];
    thread_rng().fill_bytes(&mut bytes);
    Builder::from_random_bytes(bytes).into_uuid()
}
