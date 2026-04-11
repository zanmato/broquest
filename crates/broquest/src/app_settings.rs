use crate::settings::Settings;
use gpui::{App, Global};

pub struct AppSettings {
    pub settings: Settings,
}

impl Global for AppSettings {}

impl AppSettings {
    pub fn new(_cx: &mut App, settings: Settings) -> Self {
        Self { settings }
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}
