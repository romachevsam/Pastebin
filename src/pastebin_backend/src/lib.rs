use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::{init, update, query};
use std::collections::HashMap;

type PasteId = u64;

#[derive(Clone, CandidType, Deserialize)]
struct Paste {
    id: PasteId,
    content: String,
    timestamp: u64,
}

static mut PASTE_STORE: Option<HashMap<PasteId, Paste>> = None;
static mut NEXT_ID: PasteId = 0;

#[init]
fn init() {
    unsafe {
        PASTE_STORE = Some(HashMap::new());
    }
}

#[update]
fn create_paste(content: String) -> PasteId {
    unsafe {
        if let Some(store) = PASTE_STORE.as_mut() {
            let id = NEXT_ID;
            let paste = Paste {
                id,
                content,
                timestamp: ic_cdk::api::time(),
            };
            store.insert(id, paste);
            NEXT_ID += 1;
            return id;
        }
    }
    0
}

#[query]
fn get_paste(id: PasteId) -> Option<Paste> {
    unsafe {
        if let Some(store) = PASTE_STORE.as_ref() {
            return store.get(&id).cloned();
        }
    }
    None
}

#[query]
fn list_pastes() -> Vec<Paste> {
    unsafe {
        if let Some(store) = PASTE_STORE.as_ref() {
            return store.values().cloned().collect();
        }
    }
    Vec::new()
}
