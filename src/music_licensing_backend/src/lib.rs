#[macro_use]
extern crate serde;
use ic_cdk::api::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

mod types;
use types::*;

// Define thread-local static variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static SONG_STORAGE: RefCell<StableBTreeMap<u64, Song, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static OWNER_STORAGE: RefCell<StableBTreeMap<u64, Owner, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static LICENSE_STORAGE: RefCell<StableBTreeMap<u64, License, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static LICENSEE_STORAGE: RefCell<StableBTreeMap<u64, Licensee, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));
}

// Define query functions to get all licensable songs
#[ic_cdk::query]
fn get_all_songs() -> Result<Vec<Song>, Error> {
    let songs : Vec<Song>= SONG_STORAGE.with(|s| s.borrow().iter().map(|(_, song)| song).collect());
    match songs.len() {
        0 => Err(Error::NotFound {
            msg: format!("no songs licensable could be found"),
        }),
        _ => Ok(songs),
    }
}

// Define query functions to get songs by id
#[ic_cdk::query]
fn get_song(id: u64) -> Result<Song, Error> {
    match _get_song(&id) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("song id:{} could not be found", id),
        }),
    }
}

fn _get_song(id: &u64) -> Option<Song> {
    SONG_STORAGE.with(|s| s.borrow().get(id))
}

// Define update functions to create new songs
#[ic_cdk::update]
fn create_song(payload: SongPayload) -> Result<Song, Error> {
    let mut owner =   match _get_owner(&payload.owner_id) {
        Some(owner) => Ok(owner),
        None => Err(Error::NotFound {
            msg: format!("owner id:{} could not be found", payload.owner_id),
        }),
    }?;

    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
    })
    .expect("Cannot increment Ids");

    let song = Song {
        id,
        title: payload.title.clone(),
        artist: payload.artist,
        owner_id: payload.owner_id,
        year: payload.year,
        genre: payload.genre,
        price: payload.price,
    };

    owner.song_ids.push(song.id);

    OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner.id, owner.clone()));

    SONG_STORAGE.with(|s| s.borrow_mut().insert(id, song.clone()));

    Ok(song)
}

// Define query functions to get owners by id
#[ic_cdk::query]
fn get_song_owner(id: u64) -> Result<ReturnOwner, Error> {
    let song = match _get_song(&id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", id),
            })
        }
    };

    match _get_owner(&song.owner_id) {
        Some(owner) => Ok(ReturnOwner {
            id: owner.id,
            owner_principal: owner.owner_principal,
            name: owner.name,
            email: owner.email,
        }),
        None => Err(Error::NotFound {
            msg: format!("owner id:{} could not be found", song.owner_id),
        }),
    }
}

#[ic_cdk::update]
fn update_song(payload: UpdateSongPayload) -> Result<Song, Error> {
    let song = match _get_song(&payload.id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", payload.id),
            })
        }
    };
    match check_if_caller_is_owner(&song.owner_id){
        Ok(_) => (),
        Err(e) => return Err(e)
    }

    let mut new_song = song.clone();
    new_song.title = payload.title.clone();
    new_song.artist = payload.artist;
    new_song.year = payload.year;
    new_song.genre = payload.genre;
    new_song.price = payload.price;

    SONG_STORAGE.with(|s| s.borrow_mut().insert(payload.id, new_song.clone()));
    Ok(new_song)
}

#[ic_cdk::update]
fn delete_song(id: u64) -> Result<Song, Error> {
    let song = match _get_song(&id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", id),
            })
        }
    };

    match check_if_caller_is_owner(&song.owner_id){
        Ok(_) => (),
        Err(e) => return Err(e)
    }

    match remove_song_from_owner(id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match remove_song_from_licensee(id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match SONG_STORAGE.with(|s| s.borrow_mut().remove(&id)) {
        Some(song) => Ok(song),
        None => Err(Error::InvalidPayload {
            msg: format!("song id:{} could not be deleted", id),
        }),
    }
}

fn _get_owner(id: &u64) -> Option<Owner> {
    OWNER_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn create_owner(payload: OwnerPayload) -> Result<Owner, Error> {
    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let owner = Owner {
        id,
        owner_principal: caller().to_string(),
        name: payload.name.clone(),
        email: payload.email.clone(),
        song_ids: Vec::new(),
        license_ids: Vec::new(),
    };

    OWNER_STORAGE.with(|s| s.borrow_mut().insert(id, owner.clone()));
    Ok(owner)
}

#[ic_cdk::query]
fn get_license(id: u64) -> Result<License, Error> {
    match _get_license(&id) {
        Some(license) => Ok(license),
        None => Err(Error::NotFound {
            msg: format!("license id:{} could not be found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_owner_license_requests(id: u64) -> Result<Vec<License>, Error> {
    let licenses_vec: Vec<(u64, License)> = LICENSE_STORAGE.with(|s| s.borrow().iter().collect());
    let licenses: Vec<License> = licenses_vec
        .into_iter()
        .map(|(_, license)| license)
        .collect();
    let mut owner_licenses: Vec<License> = Vec::new();

    for license in licenses {
        if license.owner_id == id {
            owner_licenses.push(license);
        }
    }

    match owner_licenses.len() {
        0 => Err(Error::NotFound {
            msg: format!("no licenses could be found for owner id:{}", id),
        }),
        _ => Ok(owner_licenses),
    }
}

#[ic_cdk::query]
fn get_licensee_licenses(id: u64) -> Result<Vec<License>, Error> {
    let licenses_vec: Vec<(u64, License)> = LICENSE_STORAGE.with(|s| s.borrow().iter().collect());
    let licenses: Vec<License> = licenses_vec
        .into_iter()
        .map(|(_, license)| license)
        .collect();
    let mut licensee_licenses: Vec<License> = Vec::new();

    for license in licenses {
        if license.licensee_id == id {
            licensee_licenses.push(license);
        }
    }

    match licensee_licenses.len() {
        0 => Err(Error::NotFound {
            msg: format!("no licenses could be found for licensee id:{}", id),
        }),
        _ => Ok(licensee_licenses),
    }
}

#[ic_cdk::update]
fn create_license_request(payload: LicensePayload) -> Result<License, Error> {
    let song = match _get_song(&payload.song_id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", payload.song_id),
            })
        }
    };

    let _is_valid_licensee_id = _get_licensee(&payload.licensee_id).ok_or_else(|| Error::NotFound { 
        msg: format!("Licensee with id={} does not exist.", payload.licensee_id) 
    })?;
    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let license = License {
        id,
        song_id: payload.song_id,
        owner_id: song.owner_id,
        licensee_id: payload.licensee_id,
        approved: false,
        price: 0,
        start_date: payload.start_date,
        end_date: payload.end_date,
    };

    LICENSE_STORAGE.with(|s| s.borrow_mut().insert(id, license.clone()));
    Ok(license)
}

#[ic_cdk::update]
fn approve_license(payload: Approvepayload) -> Result<License, Error> {
    let license = match _get_license(&payload.license_id) {
        Some(license) => license,
        None => {
            return Err(Error::NotFound {
                msg: format!("license id:{} could not be found", payload.license_id),
            })
        }
    };

    match check_if_caller_is_owner(&license.owner_id){
        Ok(_) => (),
        Err(e) => return Err(e)
    }

    let _is_valid_licensee_id = _get_licensee(&license.licensee_id).ok_or_else(|| Error::NotFound { 
        msg: format!("Licensee with id={} does not exist.", license.licensee_id) 
    })?;

    if license.approved {
        return Err(Error::AlreadyApproved {
            msg: format!(
                "license id:{} has already been approved",
                payload.license_id
            ),
        });
    }

    let mut new_license = license.clone();
    new_license.approved = true;
    new_license.price = payload.cost;

    match add_license_to_owner(license.owner_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match add_license_to_licensee(license.licensee_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    LICENSE_STORAGE.with(|s| {
        s.borrow_mut()
            .insert(payload.license_id, new_license.clone())
    });
    Ok(new_license)
}

#[ic_cdk::update]
fn revoke_license(payload: ProtectedPayload) -> Result<License, Error> {
    let license = match _get_license(&payload.license_id) {
        Some(license) => license,
        None => {
            return Err(Error::NotFound {
                msg: format!("license id:{} could not be found", payload.license_id),
            })
        }
    };

    match check_if_caller_is_owner(&license.owner_id){
        Ok(_) => (),
        Err(e) => return Err(e)
    }

    let _is_valid_licensee_id = _get_licensee(&license.licensee_id).ok_or_else(|| Error::NotFound { 
        msg: format!("Licensee with id={} does not exist.", license.licensee_id) 
    })?;

    let mut new_license = license.clone();
    new_license.approved = false;

    match remove_license_from_owner(license.owner_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match remove_license_from_licensee(license.licensee_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    LICENSE_STORAGE.with(|s| {
        s.borrow_mut()
            .insert(payload.license_id, new_license.clone())
    });
    Ok(new_license)
}

fn _get_license(id: &u64) -> Option<License> {
    LICENSE_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::query]
fn get_licensee(id: u64) -> Result<Licensee, Error> {
    match _get_licensee(&id) {
        Some(licensee) => Ok(licensee),
        None => Err(Error::NotFound {
            msg: format!("licensee id:{} could not be found", id),
        }),
    }
}

fn _get_licensee(id: &u64) -> Option<Licensee> {
    LICENSEE_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn create_licensee(payload: LicenseePayload) -> Result<Licensee, Error> {
    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let licensee = Licensee {
        id,
        name: payload.name.clone(),
        email: payload.email.clone(),
        licenses: Vec::new(),
    };

    match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(id, licensee.clone())) {
        None => Ok(licensee),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("licensee name:{} could not be created", payload.name),
        }),
    }
}

fn add_license_to_owner(owner_id: u64, license_id: u64) -> Result<(), Error> {
    let mut owner = match _get_owner(&owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", owner_id),
            })
        }
    };

    owner.license_ids.push(license_id);

    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner_id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be added to owner id:{}",
                license_id, owner_id
            ),
        }),
    }
}

fn add_license_to_licensee(licensee_id: u64, license_id: u64) -> Result<(), Error> {
    let mut licensee = match _get_licensee(&licensee_id) {
        Some(licensee) => licensee,
        None => {
            return Err(Error::NotFound {
                msg: format!("licensee id:{} could not be found", licensee_id),
            })
        }
    };

    licensee.licenses.push(license_id);

    match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(licensee_id, licensee.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be added to licensee id:{}",
                license_id, licensee_id
            ),
        }),
    }
}

fn remove_license_from_owner(owner_id: u64, license_id: u64) -> Result<(), Error> {
    let mut owner = match _get_owner(&owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", owner_id),
            })
        }
    };

    let mut index = 0;
    let mut found = false;
    for (i, id) in owner.license_ids.iter().enumerate() {
        if *id == license_id {
            index = i;
            found = true;
            break;
        }
    }

    if !found {
        return Err(Error::NotFound {
            msg: format!(
                "license id:{} could not be found in owner id:{}",
                license_id, owner_id
            ),
        });
    }

    owner.license_ids.remove(index);

    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner_id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be removed from owner id:{}",
                license_id, owner_id
            ),
        }),
    }
}

fn remove_license_from_licensee(licensee_id: u64, license_id: u64) -> Result<(), Error> {
    let mut licensee = match _get_licensee(&licensee_id) {
        Some(licensee) => licensee,
        None => {
            return Err(Error::NotFound {
                msg: format!("licensee id:{} could not be found", licensee_id),
            })
        }
    };

    let mut index = 0;
    let mut found = false;
    for (i, id) in licensee.licenses.iter().enumerate() {
        if *id == license_id {
            index = i;
            found = true;
            break;
        }
    }

    if !found {
        return Err(Error::NotFound {
            msg: format!(
                "license id:{} could not be found in licensee id:{}",
                license_id, licensee_id
            ),
        });
    }

    licensee.licenses.remove(index);

    match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(licensee_id, licensee.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be removed from licensee id:{}",
                license_id, licensee_id
            ),
        }),
    }
}

fn remove_song_from_owner(id: u64) -> Result<(), Error> {
    let song = _get_song(&id).ok_or(Error::NotFound {
        msg: format!("song id:{} could not be found", id),
    })?;

    let mut owner = _get_owner(&song.owner_id).ok_or(Error::NotFound {
        msg: format!("owner id:{} could not be found", song.owner_id),
    })?;

    let index = owner
        .song_ids
        .iter()
        .position(|&x| x == song.id)
        .ok_or(Error::NotFound {
            msg: format!(
                "song id:{} could not be found in owner id:{}",
                song.id, owner.id
            ),
        })?;

    owner.song_ids.remove(index);

    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner.id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "song id:{} could not be removed from owner id:{}",
                song.id, owner.id
            ),
        }),
    }
}

fn remove_song_from_licensee(id: u64) -> Result<(), Error> {
    let song = _get_song(&id).ok_or(Error::NotFound {
        msg: format!("song id:{} could not be found", id),
    })?;

    let licenses_vec: Vec<(u64, License)> = LICENSE_STORAGE.with(|s| s.borrow().iter().collect());
    let licenses: Vec<License> = licenses_vec
        .into_iter()
        .map(|(_, license)| license)
        .collect();
    let mut licenses_to_remove: Vec<License> = Vec::new();

    for license in licenses {
        if license.song_id == song.id {
            licenses_to_remove.push(license);
        }
    }

    for license in licenses_to_remove {
        let mut licensee = _get_licensee(&license.licensee_id).ok_or(Error::NotFound {
            msg: format!("licensee id:{} could not be found", license.licensee_id),
        })?;

        let index = licensee
            .licenses
            .iter()
            .position(|&x| x == license.id)
            .ok_or(Error::NotFound {
                msg: format!(
                    "license id:{} could not be found in licensee id:{}",
                    license.id, licensee.id
                ),
            })?;

        licensee.licenses.remove(index);

        match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(licensee.id, licensee.clone())) {
            Some(_) => (),
            None => {
                return Err(Error::InvalidPayload {
                    msg: format!(
                        "license id:{} could not be removed from licensee id:{}",
                        license.id, licensee.id
                    ),
                })
            }
        }
    }

    Ok(())
}

fn check_if_caller_is_owner(owner_id : &u64) -> Result<(), Error> {
    let owner = match _get_owner(&owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", owner_id),
            })
        }
    };

    if owner.owner_principal != caller().to_string() {
        return Err(Error::InvalidPayload {
            msg: format!(
                "Caller is not the song owner"
            ),
        });
    }else{
        Ok(())
    }
}

// Candid generator for Candid interface
ic_cdk::export_candid!();
