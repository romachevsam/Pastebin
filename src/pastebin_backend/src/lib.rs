use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::{init, update, query};
use std::collections::HashMap;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use thiserror::Error;
use ic_cdk::storage;

type PasteId = u64;

#[derive(Clone, CandidType, Deserialize)]
struct Paste {
    id: PasteId,
    content: String,
    timestamp: u64,
}

#[derive(CandidType, Deserialize, Error, Debug)]
enum PasteError {
    #[error("Paste with id {0} not found")]
    NotFound(PasteId),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl PasteError {
    fn not_found(id: PasteId) -> Self {
        PasteError::NotFound(id)
    }

    fn invalid_input(msg: &str) -> Self {
        PasteError::InvalidInput(msg.to_string())
    }
}

#[init]
fn init() {
    ic_cdk::println!("Paste management system initialized");
    storage::stable_save((HashMap::<PasteId, Paste>::new(), 0u64)).unwrap();
}

#[update]
fn create_paste(content: String) -> Result<PasteId, PasteError> {
    if content.is_empty() {
        return Err(PasteError::invalid_input("Content cannot be empty"));
    }

    let timestamp = ic_cdk::api::time();
    let (mut store, mut next_id): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();

    let new_id = next_id;
            let paste = Paste {
                id: new_id,
                content,
                timestamp,
            };

    store.insert(new_id, paste);
    next_id += 1;

    storage::stable_save((store, next_id)).unwrap();
            Ok(new_id)
}

#[query]
fn get_paste(id: PasteId) -> Result<Paste, PasteError> {
    let (store, _): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();
    store.get(&id).cloned().ok_or(PasteError::not_found(id))
}

#[update]
fn update_paste(id: PasteId, new_content: String) -> Result<Paste, PasteError> {
    if new_content.is_empty() {
        return Err(PasteError::invalid_input("Content cannot be empty"));
    }

    let timestamp = ic_cdk::api::time();
    let (mut store, next_id): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();

        if let Some(paste) = store.get_mut(&id) {
            paste.content = new_content;
        paste.timestamp = timestamp;
        storage::stable_save((store, next_id)).unwrap();
            Ok(paste.clone())
        } else {
            Err(PasteError::not_found(id))
        }
}

#[update]
fn delete_paste(id: PasteId) -> Result<Paste, PasteError> {
    let (mut store, next_id): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();
    let result = store.remove(&id).ok_or(PasteError::not_found(id));
    storage::stable_save((store, next_id)).unwrap();
    result
}

#[query]
fn list_pastes(page: Option<u64>, per_page: Option<u64>) -> Vec<Paste> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(5);

    if page == 0 || per_page == 0 {
        return Vec::new();
    }

    let (store, _): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();
        store.values()
            .cloned()
            .skip(((page - 1) * per_page) as usize)
            .take(per_page as usize)
            .collect()
}

#[query]
fn search_pastes(keyword: String) -> Vec<Paste> {
    if keyword.is_empty() {
        return Vec::new();
    }

    let (store, _): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();
    store.values()
            .filter(|paste| paste.content.contains(&keyword))
            .cloned()
            .collect()
}

#[query]
fn list_all_pastes() -> Vec<Paste> {
    let (store, _): (HashMap<PasteId, Paste>, PasteId) = storage::stable_restore().unwrap();
    store.values().cloned().collect()
}

ic_cdk::export_candid!();
