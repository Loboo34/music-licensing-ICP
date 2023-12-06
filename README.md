
# Music Licensing System

## Overview

This is a Rust implementation of a Music Licensing System Smart Contract for the Internet Computer Web3 platform. It is designed to manage songs, owners, and licenses in the context of a music licensing platform. The system provides functionalities such as creating, updating, and deleting songs, managing song ownership, and handling license requests.

## Prerequisites

- Rust
- Internet Computer SDK
- IC CDK

## Installation

1. **Clone the repository:**

    ```bash
    git clone https://github.com/loboo34/music-licensing-ICP.git
    cd music-licensing-ICP
    npm install
    dfx start --background --clean
    npm run gen-deploy
    ```

## Data Structure

### Type Aliases

- `SongStorage`: Alias for `StableBTreeMap<u64, Song>` to store songs.
- `OwnerStorage`: Alias for `StableBTreeMap<u64, Owner>` to store owners.
- `LicenseStorage`: Alias for `StableBTreeMap<u64, License>` to store licenses.
- `LicenseeStorage`: Alias for `StableBTreeMap<u64, Licensee>` to store licensees.

### Struct Definitions

- `Song`, `Owner`, `License`, `Licensee`: Structs representing Song, Owner, License, and Licensee entities.
  - Implement `CandidType`, `Clone`, `Serialize`, `Deserialize`, and provide default values.

### Trait Implementations

- `Storable` and `BoundedStorable` implemented for `Song`, `Owner`, `License`, and `Licensee`.
  - `Storable`: Conversion to and from bytes.
  - `BoundedStorable`: Defines maximum size and whether the size is fixed.

### Thread-Local Static Variables

- `MEMORY_MANAGER`: Manages virtual memory.
- `ID_COUNTER`: Keeps track of global IDs.
- `SONG_STORAGE`, `OWNER_STORAGE`, `LICENSE_STORAGE`, `LICENSEE_STORAGE`: Stable BTreeMaps for storing songs, owners, licenses, and licensees.

### Payload Structs

- `SongPayload`, `OwnerPayload`, `UpdateSongPayload`, `LicensePayload`, `ApprovePayload`, `LicenseePayload`: Payload data structures for various operations.

### Candid Interface Definitions

- Functions annotated with `ic_cdk::query` are read-only queries.
- Functions annotated with `ic_cdk::update` are updates, which can modify the state.

## Memory Management

Memory is allocated using a `MemoryManager` from the `ic-stable-structures` crate:

```rust
static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = // initialized
```

This manages allocating `VirtualMemory` for storages.

## ID Generation

Unique IDs are generated using a thread-local `IdCell`:

```rust
static ID_COUNTER: RefCell<IdCell> = // initialized
```

The counter is incremented when adding new records.

## Record Storage

Records are stored in thread-local `StableBTreeMap`s:

```rust
static SONG_STORAGE: RefCell<SongStorage> = // initialized
static OWNER_STORAGE: RefCell<OwnerStorage> = // initialized
static LICENSE_STORAGE: RefCell<LicenseStorage> = // initialized
static LICENSEE_STORAGE: RefCell<LicenseeStorage> = // initialized
```

Each storage maps IDs to their respective entities (songs, owners, licenses, and licensees).

## Main Functions

### User Functions

- `get_song(id: u64)`: Retrieve a song by ID.
- `get_all_songs()`: Retrieve all licensable songs.
- `create_song(payload: SongPayload)`: Create a new song.
- `update_song(payload: UpdateSongPayload)`: Update an existing song.
- `delete_song(auth_key: String, id: u64)`: Delete a song.

### Owner Functions

- `get_song_owner(id: u64)`: Retrieve the owner of a song.
- `create_owner(payload: OwnerPayload)`: Create a new owner.

### License Functions

- `get_license(id: u64)`: Retrieve a license by ID.
- `get_owner_license_requests(id: u64)`: Retrieve licenses requested by an owner.
- `get_licensee_licenses(id: u64)`: Retrieve licenses associated with a licensee.
- `create_license_request(payload: LicensePayload)`: Create a license request.
- `approve_license(payload: ApprovePayload)`: Approve a license.

## Error Handling

- `NotFound`: Indicates that an entity (song, owner, license) could not be found.
- `InvalidPayload`: Indicates an issue with the payload during creation or update.
- `AlreadyApproved`: Indicates an attempt to approve a license that has already been approved.
- `Unauthorized`: Indicates that the user does not have the necessary permissions.

## Learn more

To learn more before you start working with music_licensing, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd music-licensing-ICP/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `production` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
