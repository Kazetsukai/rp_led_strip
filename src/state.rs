use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use serde::{ser::SerializeTuple, Serialize};
use smart_leds::RGB8;

#[derive(Clone, Copy)]
pub struct OurRGB8 {
    inner: RGB8,
}

impl Serialize for OurRGB8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_tuple(3)?;
        s.serialize_element(&self.inner.r)?;
        s.serialize_element(&self.inner.g)?;
        s.serialize_element(&self.inner.b)?;
        s.end()
    }
}

impl From<OurRGB8> for RGB8 {
    fn from(rgb: OurRGB8) -> RGB8 {
        rgb.inner
    }
}

impl From<RGB8> for OurRGB8 {
    fn from(rgb: RGB8) -> OurRGB8 {
        OurRGB8 { inner: rgb }
    }
}

#[derive(serde::Serialize, Clone, Copy)]
pub struct LedControls {
    pub color: OurRGB8,
    pub power: bool,
}

#[derive(Clone, Copy)]
pub struct SharedState(pub &'static Mutex<CriticalSectionRawMutex, LedControls>);

pub struct AppState {
    pub shared_state: SharedState,
}

impl picoserve::extract::FromRef<AppState> for SharedState {
    fn from_ref(state: &AppState) -> Self {
        state.shared_state
    }
}
