use std::sync::atomic::{AtomicU64, Ordering};

use mmo_common::object::ObjectId;

static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(0);

pub fn next_object_id() -> ObjectId {
    ObjectId(NEXT_OBJECT_ID.fetch_add(1, Ordering::SeqCst))
}
