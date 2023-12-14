use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, Storable};
use std::borrow::Cow;

// Define type aliases for convenience
pub type Memory = VirtualMemory<DefaultMemoryImpl>;
pub type IdCell = Cell<u64, Memory>;

// Define the data structures that will be stored in the stable memory
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Song {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub owner_id: u64,
    pub year: u32, 
    pub genre: String,
    pub price: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Owner {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub owner_principal: String,
    pub song_ids: Vec<u64>,
    pub license_ids: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct License {
    pub id: u64,
    pub song_id: u64,
    pub owner_id: u64,
    pub licensee_id: u64,
    pub approved: bool,
    pub price: u32,
    pub start_date: String,
    pub end_date: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Licensee {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub licenses: Vec<u64>,
}

// Define return types for calls
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct ReturnOwner {
    pub id: u64,
    pub owner_principal: String,
    pub name: String,
    pub email: String,
}

// Implement the 'Storable' trait for each of the data structures
impl Storable for Song {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Owner {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for License {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Licensee {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the 'BoundedStorable' trait for each of the data structures
impl BoundedStorable for Song {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Owner {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for License {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Licensee {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}


// Define structs for payload data (used in update calls)
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct SongPayload {
    pub title: String,
    pub artist: String,
    pub owner_id: u64,
    pub year: u32,
    pub genre: String,
    pub price: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct OwnerPayload {
    pub name: String,
    pub email: String
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub song_id: u64,
    pub licensee_id: u64,
    pub start_date: String,
    pub end_date: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct LicenseePayload {
    pub name: String,
    pub email: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct ProtectedPayload {
    pub license_id: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct Approvepayload {
    pub license_id: u64,
    pub cost: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct UpdateSongPayload {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub genre: String,
    pub price: u32,
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
pub enum Error {
    NotFound { msg: String },
    InvalidPayload { msg: String },
    AlreadyApproved { msg: String },
}