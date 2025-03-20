#[cfg(feature = "font-kit")]
mod open_type;

#[cfg(feature = "font-kit")]
mod text_system;

mod window_appearance;

mod dispatcher;

mod window;

use crate::platform::blade as renderer;
