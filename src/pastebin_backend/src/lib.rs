#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Type alias for memory and ID cell types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;
type PasteId = u64;

// Struct representing a Paste entry
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Paste {
    id: PasteId,
    content: String,
    timestamp: u64,
}

// Implementing `Storable` for converting `Paste` to and from bytes for storage
impl Storable for Paste {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implementing `BoundedStorable` to define storage constraints for `Paste`
impl BoundedStorable for Paste {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Memory manager for stable memory, ID counter, and storage map for pastes
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static PASTE_STORAGE: RefCell<StableBTreeMap<PasteId, Paste, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

// Payload struct for creating a new Paste
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct PastePayload {
    content: String,
}

// Function to validate paste payload
fn validate_paste_payload(payload: &PastePayload) -> Result<(), String> {
    if payload.content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    Ok(())
}

// Init function for initializing memory
#[ic_cdk::init]
fn init() {
    // Memory initialization is handled by thread_local! memory management
}

// Update function to create a new paste
#[ic_cdk::update]
fn create_paste(payload: PastePayload) -> Result<PasteId, String> {
    // Validate payload
    validate_paste_payload(&payload)?;

    // Generate new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    // Create new paste
    let new_paste = Paste {
        id,
        content: payload.content,
        timestamp: time(),
    };

    // Insert paste into storage
    PASTE_STORAGE.with(|storage| storage.borrow_mut().insert(id, new_paste.clone()));

    Ok(id)
}

// Query function to get a paste by ID
#[ic_cdk::query]
fn get_paste(id: PasteId) -> Result<Paste, String> {
    match PASTE_STORAGE.with(|storage| storage.borrow().get(&id)) {
        Some(paste) => Ok(paste),
        None => Err(format!("Paste with id={} not found", id)),
    }
}

// Query function to list all pastes
#[ic_cdk::query]
fn list_pastes() -> Vec<Paste> {
    PASTE_STORAGE.with(|storage| storage.borrow().values().cloned().collect())
}

// Update function to update an existing paste
#[ic_cdk::update]
fn update_paste(id: PasteId, payload: PastePayload) -> Result<Paste, String> {
    // Validate the new content
    validate_paste_payload(&payload)?;

    // Update the paste if it exists
    PASTE_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        match storage.get(&id) {
            Some(mut existing_paste) => {
                // Update the paste content and timestamp
                existing_paste.content = payload.content;
                existing_paste.timestamp = time();

                // Insert the updated paste back into storage
                storage.insert(id, existing_paste.clone());
                Ok(existing_paste)
            }
            None => Err(format!("Paste with id={} not found", id)),
        }
    })
}

// Update function to delete an existing paste
#[ic_cdk::update]
fn delete_paste(id: PasteId) -> Result<(), String> {
    PASTE_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.remove(&id).is_some() {
            Ok(())
        } else {
            Err(format!("Paste with id={} not found", id))
        }
    })
}

// Candid export for interface generation
ic_cdk::export_candid!();
