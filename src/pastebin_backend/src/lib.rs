use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::{init, update, query};
use std::collections::HashMap;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type PasteId = u64;

#[derive(Clone, CandidType, Deserialize)]
struct Paste {
    id: PasteId,
    content: String,
    timestamp: u64,
}

// Error type for handling failures
#[derive(CandidType, Deserialize)]
enum PasteError {
    NotFound { msg: String },
    InvalidInput { msg: String },
}

impl PasteError {
    fn not_found(id: PasteId) -> Self {
        PasteError::NotFound {
            msg: format!("Paste with id {} not found", id),
        }
    }

    fn invalid_input(msg: &str) -> Self {
        PasteError::InvalidInput {
            msg: msg.to_string(),
        }
    }
}

// In-memory storage using thread_local! for thread safety
thread_local! {
    static PASTE_STORE: RefCell<HashMap<PasteId, Paste>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<PasteId> = RefCell::new(0);
}

#[init]
fn init() {
    ic_cdk::println!("Paste management system initialized");
}

// Create a new paste
#[ic_cdk::update]
fn create_paste(content: String) -> Result<PasteId, PasteError> {
    if content.is_empty() {
        return Err(PasteError::invalid_input("Content cannot be empty"));
    }

    let timestamp = ic_cdk::api::time();
    PASTE_STORE.with(|store| {
        NEXT_ID.with(|next_id| {
            let mut id_counter = next_id.borrow_mut();
            let new_id = *id_counter;

            let paste = Paste {
                id: new_id,
                content,
                timestamp,
            };

            store.borrow_mut().insert(new_id, paste);
            *id_counter += 1;
            Ok(new_id)
        })
    })
}

// Get a paste by ID
#[ic_cdk::query]
fn get_paste(id: PasteId) -> Result<Paste, PasteError> {
    PASTE_STORE.with(|store| {
        store.borrow().get(&id).cloned().ok_or(PasteError::not_found(id))
    })
}

// Update an existing paste
#[ic_cdk::update]
fn update_paste(id: PasteId, new_content: String) -> Result<Paste, PasteError> {
    if new_content.is_empty() {
        return Err(PasteError::invalid_input("Content cannot be empty"));
    }

    PASTE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        if let Some(paste) = store.get_mut(&id) {
            paste.content = new_content;
            paste.timestamp = ic_cdk::api::time();
            Ok(paste.clone())
        } else {
            Err(PasteError::not_found(id))
        }
    })
}

// Delete a paste by ID
#[ic_cdk::update]
fn delete_paste(id: PasteId) -> Result<Paste, PasteError> {
    PASTE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        store.remove(&id).ok_or(PasteError::not_found(id))
    })
}

// List all pastes with optional pagination (page and per_page parameters)
#[ic_cdk::query]
fn list_pastes(page: Option<u64>, per_page: Option<u64>) -> Vec<Paste> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(5);

    PASTE_STORE.with(|store| {
        let store = store.borrow();
        store.values()
            .cloned()
            .skip(((page - 1) * per_page) as usize)
            .take(per_page as usize)
            .collect()
    })
}

// Search pastes by keyword in the content
#[ic_cdk::query]
fn search_pastes(keyword: String) -> Vec<Paste> {
    if keyword.is_empty() {
        return Vec::new();
    }

    PASTE_STORE.with(|store| {
        store
            .borrow()
            .values()
            .filter(|paste| paste.content.contains(&keyword))
            .cloned()
            .collect()
    })
}

// List all pastes (without pagination)
#[ic_cdk::query]
fn list_all_pastes() -> Vec<Paste> {
    PASTE_STORE.with(|store| {
        store.borrow().values().cloned().collect()
    })
}

// Export candid for the canister
ic_cdk::export_candid!();
