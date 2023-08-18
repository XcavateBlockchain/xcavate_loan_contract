#![cfg_attr(not(feature = "std"), no_std)]

use ink::env::{DefaultEnvironment, Environment};
use scale::{Decode, Encode};

type AccountId = <DefaultEnvironment as Environment>::AccountId;

pub struct UniquesExtension;

impl UniquesExtension {

    pub fn create(
        origin: Origin,
        collection: u32,
        admin: AccountId,
    ) -> Result<(), UniquesError> {
        ::ink::env::chain_extension::ChainExtensionMethod::build(0x20001)
            .input::<(Origin, u32, AccountId)>()
            .output::<Result<(), UniquesError>, true>()
            .handle_error_code::<UniquesError>()
            .call(&(origin, collection, admin))
    }

    pub fn transfer(
        origin: Origin,
        collection: u32,
        item: u32,
        dest: AccountId,
    ) -> Result<(), UniquesError> {
        ::ink::env::chain_extension::ChainExtensionMethod::build(0x20001)
            .input::<(Origin, u32, u32, AccountId)>()
            .output::<Result<(), UniquesError>, true>()
            .handle_error_code::<UniquesError>()
            .call(&(origin, collection, item, dest))
    }

    pub fn burn(
        origin: Origin,
        collection: u32,
        item: u32,
        check_owner: Option<AccountId>,
    ) -> Result<(), UniquesError> {
        ::ink::env::chain_extension::ChainExtensionMethod::build(0x20001)
            .input::<(Origin, u32, u32, Option<AccountId>)>()
            .output::<Result<(), UniquesError>, true>()
            .handle_error_code::<UniquesError>()
            .call(&(origin, collection, item, check_owner))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum UniquesError {
    /// Account balance must be greater than or equal to the transfer amount.
    BalanceLow = 1,
    /// The account to alter does not exist.
    NoAccount = 2,
    /// The signing account has no permission to do the operation.
    NoPermission = 3,
    /// The given asset ID is unknown.
    Unknown = 4,
    /// The origin account is frozen.
    Frozen = 5,
    /// The asset ID is already taken.
    InUse = 6,
    /// Invalid witness data given.
    BadWitness = 7,
    /// Minimum balance should be non-zero.
    MinBalanceZero = 8,
    /// Unable to increment the consumer reference counters on the account. Either no provider
    /// reference exists to allow a non-zero balance of a non-self-sufficient asset, or the
    /// maximum number of consumers has been reached.
    NoProvider = 9,
    /// Invalid metadata given.
    BadMetadata = 10,
    /// No approval exists that would allow the transfer.
    Unapproved = 11,
    /// The source account would not survive the transfer and it needs to stay alive.
    WouldDie = 12,
    /// The asset-account already exists.
    AlreadyExists = 13,
    /// The asset-account doesn't have an associated deposit.
    NoDeposit = 14,
    /// The operation would result in funds being burned.
    WouldBurn = 15,
    /// The asset is a live asset and is actively being used. Usually emit for operations such
    /// as `start_destroy` which require the asset to be in a destroying state.
    LiveAsset = 16,
    /// The asset is not live, and likely being destroyed.
    AssetNotLive = 17,
    /// The asset status is not the expected status.
    IncorrectStatus = 18,
    /// The asset should be frozen before the given operation.
    NotFrozen = 19,
    /// Origin Caller is not supported
    OriginCannotBeCaller = 98,
    /// Unknown error
    RuntimeError = 99,
    /// Unknow status code
    UnknownStatusCode,
    /// Encountered unexpected invalid SCALE encoding
    InvalidScaleEncoding,
}

impl ink::env::chain_extension::FromStatusCode for UniquesError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::BalanceLow),
            2 => Err(Self::NoAccount),
            3 => Err(Self::NoPermission),
            4 => Err(Self::Unknown),
            5 => Err(Self::Frozen),
            6 => Err(Self::InUse),
            7 => Err(Self::BadWitness),
            8 => Err(Self::MinBalanceZero),
            9 => Err(Self::NoProvider),
            10 => Err(Self::BadMetadata),
            11 => Err(Self::Unapproved),
            12 => Err(Self::WouldDie),
            13 => Err(Self::AlreadyExists),
            14 => Err(Self::NoDeposit),
            15 => Err(Self::WouldBurn),
            16 => Err(Self::LiveAsset),
            17 => Err(Self::AssetNotLive),
            18 => Err(Self::IncorrectStatus),
            19 => Err(Self::NotFrozen),
            98 => Err(Self::OriginCannotBeCaller),
            99 => Err(Self::RuntimeError),
            _ => Err(Self::UnknownStatusCode),
        }
    }
}
impl From<scale::Error> for UniquesError {
    fn from(_: scale::Error) -> Self {
        UniquesError::InvalidScaleEncoding
    }
}

#[derive(Clone, Copy, Decode, Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Origin {
    Caller,
    Address,
}

impl Default for Origin {
    fn default() -> Self {
        Self::Address
    }
}