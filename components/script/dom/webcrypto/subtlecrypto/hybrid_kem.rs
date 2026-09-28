/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use elliptic_curve::ecdh::SharedSecret;
use elliptic_curve::point::PointCompression;
use elliptic_curve::sec1::{FromSec1Point, ModulusSize, ToSec1Point};
use elliptic_curve::{
    Curve, CurveArithmetic, PublicKey as GroupPublicKey, ScalarValue, SecretKey as GroupPrivateKey,
};
use kem::common::OutputSizeUser;
use kem::common::rand_core::{CryptoRng, TryCryptoRng};
pub(crate) use kem::{
    Ciphertext, Decapsulate, DecapsulationKey, Decapsulator, Encapsulate, EncapsulationKey,
    Generate, InvalidKey, Kem, Key, KeyExport, KeyInit, KeySizeUser, SharedKey, TryDecapsulate,
    TryKeyInit,
};
use ml_kem::array::Array;
use ml_kem::array::sizes::{U32, U48, U128, U1153, U1249, U1665};
use ml_kem::array::typenum::Unsigned;
use ml_kem::{
    ArraySize, EncapsulationKey768 as MlKem768EncapsulationKey,
    EncapsulationKey1024 as MlKem1024EncapsulationKey, MlKem768, MlKem1024,
};
use p256::NistP256;
use p384::NistP384;
use sha3::{Digest, Sha3_256};
use shake::digest::{ExtendableOutput, XofReader};
use shake::{Shake256, Update};
use zeroize::ZeroizeOnDrop;

/// MLKEM768-P256 Key Encapsulation Mechanisms.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct MlKem768P256 {}

/// MLKEM768-X25519 Key Encapsulation Mechanisms.
pub(crate) use x_wing::XWingKem as MlKem768X25519;

/// MLKEM1024-P384 Key Encapsulation Mechanisms.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct MlKem1024P384 {}

/// Error type for ['TryDecapsulate'] for cases whose the decapsulation fails.
#[derive(Debug)]
pub(crate) struct DecapsulationError;

impl core::fmt::Display for DecapsulationError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_str("Decapsulation failed")
    }
}

impl core::error::Error for DecapsulationError {}

impl From<std::array::TryFromSliceError> for DecapsulationError {
    fn from(_value: std::array::TryFromSliceError) -> Self {
        DecapsulationError
    }
}

/// KEM components and constant of the concrete hybrid KEM instances, specified in Section 4 of
/// draft-irtf-cfrg-concrete-hybrid-kems-04.
///
/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-4>
pub(crate) trait HybridKemParameter
where
    <Self::GroupT as Curve>::FieldBytesSize: ModulusSize,
    <Self::GroupT as CurveArithmetic>::AffinePoint:
        FromSec1Point<Self::GroupT> + ToSec1Point<Self::GroupT>,
    <Self::KemPQ as Kem>::EncapsulationKey: EncapsulateDeterministic,
    <Self::KemPQ as Kem>::DecapsulationKey: Decapsulate + KeyInit,
{
    /// `Group_T` component
    type GroupT: Curve + CurveArithmetic + PointCompression + RandomScalar;
    /// `KEM_PQ` component
    type KemPQ: Kem;
    /// `PRG` component
    type PRG: Default + Update + ExtendableOutput;
    /// `KDF` component
    type KDF: Default + Digest;
    /// `Label` component
    const LABEL: &[u8];

    /// `Nseed` constant. The length of seed.
    type SeedSize: ArraySize;
    /// `Nek` constant. The length of encapsulation key.
    type EncapsulationKeySize: ArraySize;
    /// `Ndk` constant. The length of decapsulation key.
    type DecapsulationKeySize: ArraySize;
    /// `Nct` constant. The length of ciphertext key.
    type CiphertextSize: ArraySize;
    /// `Nss` constant. The length of shared secret key.
    type SharedSecretSize: ArraySize;

    // NOTE: The seed is directly used as the decapsulation key, so the seed length must be same as
    // the decapsulation key length. However, Rust compiler does not know it from this trait
    // definition. In this implementation, we directly use `DecapsulationKeySize` instead of
    // `SeedSize` when we work with seed.
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-4.1>
impl HybridKemParameter for MlKem768P256 {
    type GroupT = NistP256;
    type KemPQ = MlKem768;
    type PRG = Shake256;
    type KDF = Sha3_256;
    const LABEL: &[u8] = br"MLKEM768-P256";

    type SeedSize = U32;
    type EncapsulationKeySize = U1249;
    type DecapsulationKeySize = U32;
    type CiphertextSize = U1153;
    type SharedSecretSize = U32;
}

impl Kem for MlKem768P256 {
    type DecapsulationKey = HybridKemDecapsulationKey<Self>;
    type EncapsulationKey = HybridKemEncapsulationKey<Self>;
    type SharedKeySize = <Self as HybridKemParameter>::SharedSecretSize;
    type CiphertextSize = <Self as HybridKemParameter>::CiphertextSize;
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-4.3>
impl HybridKemParameter for MlKem1024P384 {
    type GroupT = NistP384;
    type KemPQ = MlKem1024;
    type PRG = Shake256;
    type KDF = Sha3_256;
    const LABEL: &[u8] = br"MLKEM1024-P384";

    type SeedSize = U32;
    type EncapsulationKeySize = U1665;
    type DecapsulationKeySize = U32;
    type CiphertextSize = U1665;
    type SharedSecretSize = U32;
}

impl Kem for MlKem1024P384 {
    type DecapsulationKey = HybridKemDecapsulationKey<Self>;
    type EncapsulationKey = HybridKemEncapsulationKey<Self>;
    type SharedKeySize = <Self as HybridKemParameter>::SharedSecretSize;
    type CiphertextSize = <Self as HybridKemParameter>::CiphertextSize;
}

// Naming convention
//
// seed: random seed
// ss  : shared secret
// ct  : ciphertext
// ek  : encapsulation key (encapsulation key for KEMs, public key for Nominal Groups)
// dk  : decapsulation key (decapsulation key for KEMs, secret key for Nominal Groups)
// sk  : secret key for Nominal Groups
//
// _PQ : Post-quantum
// _T  : Traditional

/// A hybrid KEM encapsulation key whose the KEM components and constants are specified by
/// [`HybridKemParameter`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridKemEncapsulationKey<H: HybridKemParameter> {
    encapsulation_key_pq: <H::KemPQ as Kem>::EncapsulationKey,
    encapsulation_key_t: GroupPublicKey<H::GroupT>,
}

impl<H: HybridKemParameter + Kem> HybridKemEncapsulationKey<H> {
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.5>
    ///
    /// Encaps(ek):
    fn encapsulate_deterministic(
        &self,
        randomness_pq: &Array<
            u8,
            <<H::KemPQ as Kem>::EncapsulationKey as EncapsulateDeterministic>::SeedSize,
        >,
        randomness_t: &Array<u8, <H::GroupT as RandomScalar>::SeedSize>,
    ) -> (SharedKey<H>, Ciphertext<H>) {
        // (ek_PQ, ek_T) = split(KEM_PQ.Nek, Group_T.Nelem, ek)
        let encapsulation_key_pq = &self.encapsulation_key_pq;
        let encapsulation_key_t = self.encapsulation_key_t;

        // (ss_PQ, ss_T, ct_PQ, ct_T) = prepareEncapsG(ek_PQ, ek_T)
        let (shared_secret_pq, shared_secret_t, ciphertext_pq, ciphertext_t) = prepare_encaps_g::<H>(
            encapsulation_key_pq,
            &encapsulation_key_t,
            randomness_pq,
            randomness_t,
        );

        // ss_H = C2PRICombiner(ss_PQ, ss_T, ct_T, ek_T, Label)
        let shared_secret_h = c2pri_combiner::<H>(
            &shared_secret_pq,
            &shared_secret_t,
            &ciphertext_t,
            &encapsulation_key_t,
            H::LABEL,
        );

        // ct_H = concat(ct_PQ, ct_T)
        let mut ciphertext_h = Array::default();
        let (ciphertext_h_left, ciphertext_h_right) =
            ciphertext_h.split_at_mut(<H::KemPQ as Kem>::CiphertextSize::USIZE);
        ciphertext_h_left.copy_from_slice(&ciphertext_pq);
        ciphertext_h_right.copy_from_slice(&ciphertext_t.to_sec1_bytes());

        // return (ss_H, ct_H)
        (
            Array::try_from(shared_secret_h.as_slice())
                .expect("The length of shared secret must match the output length of KDF"),
            ciphertext_h,
        )
    }
}

impl<H: HybridKemParameter + Kem> Encapsulate for HybridKemEncapsulationKey<H> {
    type Kem = H;

    fn encapsulate_with_rng<R>(&self, rng: &mut R) -> (Ciphertext<Self::Kem>, SharedKey<Self::Kem>)
    where
        R: CryptoRng + ?Sized,
    {
        let mut randomness_pq = Array::default();
        let mut randomness_t = Array::default();
        rng.try_fill_bytes(randomness_pq.as_mut_slice());
        rng.try_fill_bytes(randomness_t.as_mut_slice());
        let (shared_secret, ciphertext) =
            self.encapsulate_deterministic(&randomness_pq, &randomness_t);
        (ciphertext, shared_secret)
    }
}

impl<H: HybridKemParameter> KeyExport for HybridKemEncapsulationKey<H> {
    fn to_bytes(&self) -> Key<Self> {
        // The encapsulation key of hybrid KEM is the concatenation of the encapsulation key of
        // post-quantum component and the encapsulation key of traditional component.
        let mut bytes = Key::<Self>::default();
        let (bytes_pq, bytes_t) =
            bytes.split_at_mut(<H::KemPQ as Kem>::EncapsulationKey::key_size());
        bytes_pq.copy_from_slice(&self.encapsulation_key_pq.to_bytes());
        bytes_t.copy_from_slice(&self.encapsulation_key_t.to_sec1_bytes());
        bytes
    }
}

impl<H: HybridKemParameter> TryKeyInit for HybridKemEncapsulationKey<H> {
    fn new(key: &Key<Self>) -> Result<Self, InvalidKey> {
        // The encapsulation key of hybrid KEM is the concatenation of the encapsulation key of
        // post-quantum component and the encapsulation key of traditional component.
        let (bytes_pq, bytes_t) = key.split_at(<H::KemPQ as Kem>::EncapsulationKey::key_size());
        let encapsulation_key_pq = <H::KemPQ as Kem>::EncapsulationKey::new_from_slice(bytes_pq)?;
        let encapsulation_key_t =
            GroupPublicKey::<H::GroupT>::from_sec1_bytes(bytes_t).map_err(|_| InvalidKey)?;
        Ok(HybridKemEncapsulationKey {
            encapsulation_key_pq,
            encapsulation_key_t,
        })
    }
}

impl<H: HybridKemParameter> KeySizeUser for HybridKemEncapsulationKey<H> {
    type KeySize = H::EncapsulationKeySize;
}

/// A hybrid KEM decapsulation key whose the KEM components and constants are specified by
/// [`HybridKemParameter`].
pub(crate) struct HybridKemDecapsulationKey<H: HybridKemParameter + Kem> {
    seed: Array<u8, H::DecapsulationKeySize>,
    encapsulation_key: <H as Kem>::EncapsulationKey,
}

impl<H: HybridKemParameter + Kem> HybridKemDecapsulationKey<H> {
    /// Private key as bytes.
    pub(crate) fn as_bytes(&self) -> &Array<u8, <H as HybridKemParameter>::DecapsulationKeySize> {
        &self.seed
    }
}

impl<H: HybridKemParameter + Kem> TryDecapsulate for HybridKemDecapsulationKey<H> {
    type Error = DecapsulationError;

    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.5>
    ///
    /// Decaps(dk, ct):
    fn try_decapsulate(
        &self,
        ciphertext: &Ciphertext<Self::Kem>,
    ) -> Result<SharedKey<Self::Kem>, Self::Error> {
        // (ct_PQ, ct_T) = split(KEM_PQ.Nct, Group_T.Nelem, ct)
        let (ciphertext_pq, ciphertext_t) =
            ciphertext.split_at(<H::KemPQ as Kem>::CiphertextSize::USIZE);
        let ciphertext_pq = Array::slice_as_array(ciphertext_pq).ok_or(DecapsulationError)?;
        // let ciphertext_t = Array::slice_as_array(ciphertext_t).ok_or(DecapsulationError)?;
        let ciphertext_t =
            GroupPublicKey::from_sec1_bytes(ciphertext_t).map_err(|_| DecapsulationError)?;

        // (ek_PQ, ek_T, dk_PQ, dk_T) = expandDecapsKeyG(dk)
        let (_encapsulation_key_pq, encapsulation_key_t, decapsulation_key_pq, decapsulation_key_t) =
            expand_decaps_key_g::<H>(&self.seed);

        // (ss_PQ, ss_T) = prepareDecapsG(ct_PQ, ct_T, dk_PQ, dk_T)
        let (shared_secret_pq, shared_secret_t) = prepare_decaps_g::<H>(
            ciphertext_pq,
            &ciphertext_t,
            &decapsulation_key_pq,
            &decapsulation_key_t,
        );

        // ss_H = C2PRICombiner(ss_PQ, ss_T, ct_T, ek_T, Label)
        let shared_secret_h = c2pri_combiner::<H>(
            &shared_secret_pq,
            &shared_secret_t,
            &ciphertext_t,
            &encapsulation_key_t,
            H::LABEL,
        );

        // return ss_H
        Ok(Array::try_from(shared_secret_h.as_slice())
            .expect("The length of shared secret must match the output length of KDF"))
    }
}

impl<H: HybridKemParameter + Kem> Decapsulator for HybridKemDecapsulationKey<H> {
    type Kem = H;

    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.2>
    ///
    /// DecapsToEncaps(dk)
    fn encapsulation_key(&self) -> &EncapsulationKey<Self::Kem> {
        // # The dk is always in seed format
        // (_, ek) = DeriveKeyPair(dk)
        // return ek
        //
        // NOTE: The encapsulation key is pre-computed by HybridKemDecapsulationKey<H>::new.
        &self.encapsulation_key
    }
}

impl<H: HybridKemParameter + Kem> Generate for HybridKemDecapsulationKey<H> {
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.2>
    ///
    /// GenerateKeyPair()
    fn try_generate_from_rng<R: TryCryptoRng + ?Sized>(rng: &mut R) -> Result<Self, R::Error> {
        // seed = random(Nseed)
        let seed = Array::try_generate_from_rng(rng)?;

        // return DeriveKeyPair(seed)
        Ok(HybridKemDecapsulationKey::new(&seed))
    }
}

impl<H: HybridKemParameter + Kem> KeyInit for HybridKemDecapsulationKey<H> {
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.5>
    ///
    /// DeriveKeyPair(seed):
    fn new(seed: &Key<Self>) -> Self {
        // (ek_PQ, ek_T, dk_PQ, dk_T) = expandDecapsKeyG(seed)
        let (
            encapsulation_key_pq,
            encapsulation_key_t,
            _decapsulation_key_pq,
            _decapsulation_key_t,
        ) = expand_decaps_key_g::<H>(seed);

        // return (seed, concat(ek_PQ, ek_T))
        let mut concatenated_encapsulation_key = Array::default();
        let (part_pq, part_t) = concatenated_encapsulation_key
            .split_at_mut(<H::KemPQ as Kem>::EncapsulationKey::key_size());
        part_pq.copy_from_slice(&encapsulation_key_pq.to_bytes());
        part_t.copy_from_slice(&encapsulation_key_t.to_sec1_bytes());
        HybridKemDecapsulationKey {
            seed: seed.clone(),
            encapsulation_key: <H as Kem>::EncapsulationKey::new(&concatenated_encapsulation_key)
                .expect("Reconstructed valid encapsulation key should remain valid"),
        }
    }
}

impl<H: HybridKemParameter + Kem> KeySizeUser for HybridKemDecapsulationKey<H> {
    type KeySize = H::DecapsulationKeySize;
}

impl<H: HybridKemParameter + Kem> ZeroizeOnDrop for HybridKemDecapsulationKey<H> {}

/// A trait for providing common interface to encapsulate with given randomness.
pub(crate) trait EncapsulateDeterministic: Encapsulate {
    type SeedSize: ArraySize;

    /// Encapsulates with the given randomness.
    fn encapsulate_deterministic(
        &self,
        seed: &Array<u8, Self::SeedSize>,
    ) -> (Ciphertext<Self::Kem>, SharedKey<Self::Kem>);
}

impl EncapsulateDeterministic for MlKem768EncapsulationKey {
    type SeedSize = U32;

    fn encapsulate_deterministic(
        &self,
        seed: &Array<u8, Self::SeedSize>,
    ) -> (Ciphertext<Self::Kem>, SharedKey<Self::Kem>) {
        self.encapsulate_deterministic(seed)
    }
}

impl EncapsulateDeterministic for MlKem1024EncapsulationKey {
    type SeedSize = U32;

    fn encapsulate_deterministic(
        &self,
        seed: &Array<u8, Self::SeedSize>,
    ) -> (Ciphertext<Self::Kem>, SharedKey<Self::Kem>) {
        self.encapsulate_deterministic(seed)
    }
}

/// Error type for [`RandomScalar::random_scalar'].
#[derive(Debug)]
pub(crate) struct RejectionSamplingError;

impl core::fmt::Display for RejectionSamplingError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_str("Rejection sampling failed")
    }
}

impl core::error::Error for RejectionSamplingError {}

/// A trait for implementing the `RandomScalar(seed)` algorithm for nominal groups, as described in
/// Section 3.1 of draft-irtf-cfrg-concrete-hybrid-kems-04.
///
/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-3.1.1>
pub(crate) trait RandomScalar: Curve {
    /// Length of `seed`.
    type SeedSize: ArraySize;

    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-3.1.1>
    ///
    /// RandomScalar(seed)
    fn random_scalar(
        seed: &Array<u8, Self::SeedSize>,
    ) -> Result<GroupPrivateKey<Self>, RejectionSamplingError> {
        // start = 0
        // end = Nscalar
        // sk = OS2IP(seed[start : end])
        //
        // while sk == 0 || sk >= order:
        //   start = end
        //   end = end + Nscalar
        //   if end > len(seed):
        //       raise Exception("Rejection sampling failed")
        //   sk = OS2IP(seed[start : end])
        // return sk
        #[expect(clippy::chunks_exact_to_as_chunks)]
        for chunk in seed.chunks_exact(Self::FieldBytesSize::USIZE) {
            if let Some(secret_key) = Array::try_from(chunk)
                .ok()
                .and_then(|bytes| ScalarValue::from_bytes(&bytes).into_option())
                .and_then(|scalar| GroupPrivateKey::from_scalar(scalar).into_option())
            {
                return Ok(secret_key);
            }
        }
        Err(RejectionSamplingError)
    }
}

impl RandomScalar for NistP256 {
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-3.1.1>
    ///
    /// Group_T.Nseed for P-256
    type SeedSize = U128;
}

impl RandomScalar for NistP384 {
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-concrete-hybrid-kems-04.html#section-3.1.1>
    ///
    /// Group_T.Nseed for P-384
    type SeedSize = U48;
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.1.1>
///
/// expandDecapsKeyG(seed)
#[expect(clippy::type_complexity)]
fn expand_decaps_key_g<H: HybridKemParameter>(
    seed: &Array<u8, H::DecapsulationKeySize>,
) -> (
    <H::KemPQ as Kem>::EncapsulationKey,
    GroupPublicKey<H::GroupT>,
    <H::KemPQ as Kem>::DecapsulationKey,
    GroupPrivateKey<H::GroupT>,
) {
    // seed_full = PRG(seed)
    let mut prg = H::PRG::default();
    prg.update(seed);
    let mut seed_full = prg.finalize_xof();

    // (seed_PQ, seed_T) = split(KEM_PQ.Nseed, Group_T.Nseed, seed_full)
    let mut seed_pq = Array::default();
    let mut seed_t = Array::default();
    seed_full.read(&mut seed_pq);
    seed_full.read(&mut seed_t);

    // (dk_PQ, ek_PQ) = KEM_PQ.DeriveKeyPair(seed_PQ)
    let decapsulation_key_pq = <H::KemPQ as Kem>::DecapsulationKey::new(&seed_pq);
    let encapsulation_key_pq = decapsulation_key_pq.encapsulation_key().clone();

    // dk_T = Group_T.RandomScalar(seed_T)
    let decapsulation_key_t = H::GroupT::random_scalar(&seed_t)
        .expect("RandomScalar fails with cryptographically negligible probability");

    // ek_T = Group_T.Exp(Group_T.g, dk_T)
    let encapsulation_key_t = decapsulation_key_t.public_key();

    // return (ek_PQ, ek_T, dk_PQ, dk_T)
    (
        encapsulation_key_pq,
        encapsulation_key_t,
        decapsulation_key_pq,
        decapsulation_key_t,
    )
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.1.1>
///
/// prepareEncapsG(ek_PQ, ek_T)
#[expect(clippy::type_complexity)]
fn prepare_encaps_g<H: HybridKemParameter>(
    encapsulation_key_pq: &<H::KemPQ as Kem>::EncapsulationKey,
    encapsulation_key_t: &GroupPublicKey<H::GroupT>,
    randomness_pq: &Array<
        u8,
        <<H::KemPQ as Kem>::EncapsulationKey as EncapsulateDeterministic>::SeedSize,
    >,
    randomness_t: &Array<u8, <H::GroupT as RandomScalar>::SeedSize>,
) -> (
    Array<u8, <H::KemPQ as Kem>::SharedKeySize>,
    Array<u8, <H::GroupT as Curve>::FieldBytesSize>,
    Array<u8, <H::KemPQ as Kem>::CiphertextSize>,
    GroupPublicKey<H::GroupT>,
) {
    // (ss_PQ, ct_PQ) = KEM_PQ.Encaps(ek_PQ)
    let (ciphertext_pq, shared_secret_pq) =
        encapsulation_key_pq.encapsulate_deterministic(randomness_pq);
    // H::encapsulate_deterministic_pq(encapsulation_key_pq, randomness_pq);

    // sk_E = Group_T.RandomScalar(random(Group_T.Nseed))
    let secret_key_e = H::GroupT::random_scalar(randomness_t)
        .expect("RandomScalar fails with cryptographically negligible probability");

    // ct_T = Group_T.Exp(Group_T.g, sk_E)
    let ciphertext_t = secret_key_e.public_key();

    // ss_T = Group_T.ElementToSharedSecret(Group_T.Exp(ek_T, sk_E))
    let shared_secret_t =
        element_to_shared_secret::<H>(secret_key_e.diffie_hellman(encapsulation_key_t));

    // return (ss_PQ, ss_T, ct_PQ, ct_T)
    (
        shared_secret_pq,
        shared_secret_t,
        ciphertext_pq,
        ciphertext_t,
    )
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.1.1>
///
/// prepareDecapsG(ct_PQ, ct_T, dk_PQ, dk_T)
#[expect(clippy::type_complexity)]
fn prepare_decaps_g<H: HybridKemParameter>(
    ciphertext_pq: &Array<u8, <H::KemPQ as Kem>::CiphertextSize>,
    ciphertext_t: &GroupPublicKey<H::GroupT>,
    decapsulation_key_pq: &<H::KemPQ as Kem>::DecapsulationKey,
    decapsulation_key_t: &GroupPrivateKey<H::GroupT>,
) -> (
    Array<u8, <H::KemPQ as Kem>::SharedKeySize>,
    Array<u8, <H::GroupT as Curve>::FieldBytesSize>,
) {
    // ss_PQ = KEM_PQ.Decaps(dk_PQ, ct_PQ)
    let shared_secret_pq = decapsulation_key_pq.decapsulate(ciphertext_pq);

    // ss_T = Group_T.ElementToSharedSecret(Group_T.Exp(ct_T, dk_T))
    let shared_secret_t =
        element_to_shared_secret::<H>(decapsulation_key_t.diffie_hellman(ciphertext_t));

    // return (ss_PQ, ss_T)
    (shared_secret_pq, shared_secret_t)
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#section-5.1.3>
///
/// C2PRICombiner(ss_PQ, ss_T, ct_T, ek_T, label)
fn c2pri_combiner<H: HybridKemParameter>(
    shared_secret_pq: &SharedKey<H::KemPQ>,
    shared_secret_t: &Array<u8, <H::GroupT as Curve>::FieldBytesSize>,
    ciphertext_t: &GroupPublicKey<H::GroupT>,
    encapsulation_key_t: &GroupPublicKey<H::GroupT>,
    label: &[u8],
) -> Array<u8, <H::KDF as OutputSizeUser>::OutputSize> {
    // return KDF(concat(ss_PQ, ss_T, ct_T, ek_T, label))
    let mut hasher = H::KDF::default();
    hasher.update(shared_secret_pq);
    hasher.update(shared_secret_t);
    hasher.update(ciphertext_t.to_sec1_bytes());
    hasher.update(encapsulation_key_t.to_sec1_bytes());
    hasher.update(label);
    hasher.finalize()
}

/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hybrid-kems-12.html#name-nominal-groups>
///
/// ElementToSharedSecret(P) -> ss
fn element_to_shared_secret<H: HybridKemParameter>(
    p: SharedSecret<H::GroupT>,
) -> Array<u8, <H::GroupT as Curve>::FieldBytesSize> {
    // Extract a shared secret from an element of the group (e.g., by taking the X coordinate of an
    // elliptic curve point).
    *p.raw_secret_bytes()
}
