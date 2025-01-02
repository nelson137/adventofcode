use std::{
    collections::HashSet,
    sync::{LazyLock, RwLock, atomic::AtomicBool},
};

use super::super::{Cursor, Direction, Map};

pub(super) static DID_MAP_CHANGE: AtomicBool = AtomicBool::new(true);

pub(super) struct AppState_ {
    pub(super) map: Map,
    pub(super) cursor: Cursor,
    pub(super) direction: Direction,
    pub(super) probe_succeeded: bool,
}

impl AppState_ {
    const fn empty() -> Self {
        Self {
            map: Map::empty(),
            cursor: Cursor::new(0, 0),
            direction: Direction::North,
            probe_succeeded: false,
        }
    }
}

pub(super) static APP_STATE: RwLock<AppState_> = RwLock::new(AppState_::empty());

pub(super) static PATH: LazyLock<RwLock<HashSet<(Cursor, Direction)>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

pub(super) static POINTER_LOCATION: RwLock<Cursor> = RwLock::new(Cursor::zero());
