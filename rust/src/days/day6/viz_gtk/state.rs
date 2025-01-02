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

// glib::wrapper! {
//     pub(super) struct AppState(ObjectSubclass<state_imp::AppState>);
// }
//
// impl AppState {
//     pub(super) fn new() -> Self {
//         glib::Object::builder().build()
//     }
// }
//
// impl Default for AppState {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// mod state_imp {
//     use std::cell::RefCell;
//
//     use gtk::glib::{self, prelude::*, subclass::prelude::*};
//
//     #[derive(Default, glib::Properties)]
//     #[properties(wrapper_type = super::AppState)]
//     pub(in super::super) struct AppState {
//         #[property(get, set)]
//         cursor: RefCell<super::MapCursor>,
//     }
//
//     #[glib::object_subclass]
//     impl ObjectSubclass for AppState {
//         const NAME: &'static str = "AdventOfCodeDay6Part2VizState";
//         type Type = super::AppState;
//         type ParentType = glib::Object;
//     }
//
//     #[glib::derived_properties]
//     impl ObjectImpl for AppState {}
// }
//
// glib::wrapper! {
//     pub(super) struct MapCursor(ObjectSubclass<map_cursor_imp::MapCursor>);
// }
//
// impl MapCursor {
//     #[allow(clippy::new_without_default)]
//     pub(super) fn new() -> Self {
//         glib::Object::builder().build()
//     }
//
//     pub(super) fn to_parts(&self) -> (Cursor, Direction) {
//         let cursor = Cursor::new(self.row() as usize, self.col() as usize);
//         let direction = self.direction().into();
//         (cursor, direction)
//     }
// }
//
// impl Default for MapCursor {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// mod map_cursor_imp {
//     use std::cell::Cell;
//
//     use gtk::glib::{self, prelude::*};
//     use gtk::subclass::prelude::*;
//
//     #[derive(Default, glib::Properties)]
//     #[properties(wrapper_type = super::MapCursor)]
//     pub(in super::super) struct MapCursor {
//         #[property(get, set)]
//         row: Cell<i64>,
//
//         #[property(get, set)]
//         col: Cell<i64>,
//
//         #[property(get, set, default = super::Direction::North as u8)]
//         direction: Cell<u8>,
//     }
//
//     #[glib::object_subclass]
//     impl ObjectSubclass for MapCursor {
//         const NAME: &'static str = "AdventOfCodeDay6Part2VizMapCursor";
//         type Type = super::MapCursor;
//         type ParentType = glib::Object;
//     }
//
//     #[glib::derived_properties]
//     impl ObjectImpl for MapCursor {}
// }
//
// pub(super) trait CursorExt {
//     fn to_map_cursor(self, direction: Direction) -> MapCursor;
// }
//
// impl CursorExt for Cursor {
//     fn to_map_cursor(self, direction: Direction) -> MapCursor {
//         let map_cursor = MapCursor::default();
//         map_cursor.set_row(self.row as i64);
//         map_cursor.set_col(self.col as i64);
//         map_cursor.set_direction(direction as u8);
//         map_cursor
//     }
// }

// glib::wrapper! {
//     pub struct CellHighlight(ObjectSubclass<cell_highlight_imp::CellHighlight>)
//         @extends gtk::Widget,
//         @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
// }
//
// impl CellHighlight {
//     pub fn new() -> Self {
//         glib::Object::builder().build()
//     }
// }
//
// impl Default for CellHighlight {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// mod cell_highlight_imp {
//     use gtk::{glib, subclass::prelude::*};
//
//     #[derive(Default)]
//     pub struct CellHighlight;
//
//     #[glib::object_subclass]
//     impl ObjectSubclass for CellHighlight {
//         const NAME: &'static str = "MyGtkAppCustomButton";
//         type Type = super::CellHighlight;
//         type ParentType = gtk::Widget;
//     }
//
//     impl ObjectImpl for CellHighlight {}
//
//     impl WidgetImpl for CellHighlight {}
// }
