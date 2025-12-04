use std::{
    collections::HashSet,
    sync::{LazyLock, RwLock, atomic::AtomicBool},
};

use super::super::{Cursor, Map, Pos};

pub(super) static DID_MAP_CHANGE: AtomicBool = AtomicBool::new(true);

pub(super) struct AppState_ {
    pub(super) map: Map,
    pub(super) cursor: Cursor,
    pub(super) probe_succeeded: bool,
}

impl AppState_ {
    const fn empty() -> Self {
        Self {
            map: Map::empty(),
            cursor: Cursor::DEFAULT,
            probe_succeeded: false,
        }
    }
}

pub(super) static APP_STATE: RwLock<AppState_> = RwLock::new(AppState_::empty());

pub(super) static PATH: LazyLock<RwLock<HashSet<Cursor>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

pub(super) static POINTER_POSITION: RwLock<Pos> = RwLock::new(Pos::ZERO);
