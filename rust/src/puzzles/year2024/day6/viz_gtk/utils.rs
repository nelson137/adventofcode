use std::fmt;

use gtk::{gdk, glib};

pub(super) fn eat_err(r: anyhow::Result<()>) {
    if let Err(err) = r {
        glib::g_error!("viz", "{err}");
    }
}

#[allow(dead_code)]
pub(super) struct K_(pub(super) gdk::Key, pub(super) gdk::ModifierType);

impl fmt::Debug for K_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.name() {
            Some(name) => write!(f, "{name}")?,
            None => write!(f, "(no-name)")?,
        }
        if !self.1.is_empty() {
            write!(f, " {:?}", self.1)?;
        }
        Ok(())
    }
}
