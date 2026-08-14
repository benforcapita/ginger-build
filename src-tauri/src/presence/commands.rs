// Ginger Code — Presence Tauri commands
use crate::presence::{GingerPresence, GingerState, GingerConfig, GingerMessage, Personality};
use tauri::State;

#[tauri::command]
pub fn presence_state(presence: State<'_, GingerPresence>) -> GingerState {
    presence.state()
}

#[tauri::command]
pub fn presence_set_state(presence: State<'_, GingerPresence>, state: GingerState) {
    presence.set_state(state);
}

#[tauri::command]
pub fn presence_config(presence: State<'_, GingerPresence>) -> GingerConfig {
    presence.config()
}

#[tauri::command]
pub fn presence_set_config(presence: State<'_, GingerPresence>, config: GingerConfig) {
    presence.set_config(config);
}

#[tauri::command]
pub fn presence_message(presence: State<'_, GingerPresence>) -> Option<GingerMessage> {
    presence.message()
}

#[tauri::command]
pub fn presence_toggle_commentary(presence: State<'_, GingerPresence>) {
    presence.toggle_commentary();
}

#[tauri::command]
pub fn presence_cycle_personality(presence: State<'_, GingerPresence>) -> Personality {
    presence.cycle_personality();
    presence.config().personality
}