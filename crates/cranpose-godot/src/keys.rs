use cranpose::prelude::{KeyCode, KeyEvent, KeyEventType, Modifiers};
use godot::{
    classes::InputEventKey,
    global::{Key, KeyLocation},
    obj::Gd,
};

const KEYS: [(Key, KeyCode); 74] = [
    (Key::A, KeyCode::A),
    (Key::B, KeyCode::B),
    (Key::C, KeyCode::C),
    (Key::D, KeyCode::D),
    (Key::E, KeyCode::E),
    (Key::F, KeyCode::F),
    (Key::G, KeyCode::G),
    (Key::H, KeyCode::H),
    (Key::I, KeyCode::I),
    (Key::J, KeyCode::J),
    (Key::K, KeyCode::K),
    (Key::L, KeyCode::L),
    (Key::M, KeyCode::M),
    (Key::N, KeyCode::N),
    (Key::O, KeyCode::O),
    (Key::P, KeyCode::P),
    (Key::Q, KeyCode::Q),
    (Key::R, KeyCode::R),
    (Key::S, KeyCode::S),
    (Key::T, KeyCode::T),
    (Key::U, KeyCode::U),
    (Key::V, KeyCode::V),
    (Key::W, KeyCode::W),
    (Key::X, KeyCode::X),
    (Key::Y, KeyCode::Y),
    (Key::Z, KeyCode::Z),
    (Key::KEY_0, KeyCode::Digit0),
    (Key::KEY_1, KeyCode::Digit1),
    (Key::KEY_2, KeyCode::Digit2),
    (Key::KEY_3, KeyCode::Digit3),
    (Key::KEY_4, KeyCode::Digit4),
    (Key::KEY_5, KeyCode::Digit5),
    (Key::KEY_6, KeyCode::Digit6),
    (Key::KEY_7, KeyCode::Digit7),
    (Key::KEY_8, KeyCode::Digit8),
    (Key::KEY_9, KeyCode::Digit9),
    (Key::F1, KeyCode::F1),
    (Key::F2, KeyCode::F2),
    (Key::F3, KeyCode::F3),
    (Key::F4, KeyCode::F4),
    (Key::F5, KeyCode::F5),
    (Key::F6, KeyCode::F6),
    (Key::F7, KeyCode::F7),
    (Key::F8, KeyCode::F8),
    (Key::F9, KeyCode::F9),
    (Key::F10, KeyCode::F10),
    (Key::F11, KeyCode::F11),
    (Key::F12, KeyCode::F12),
    (Key::UP, KeyCode::ArrowUp),
    (Key::DOWN, KeyCode::ArrowDown),
    (Key::LEFT, KeyCode::ArrowLeft),
    (Key::RIGHT, KeyCode::ArrowRight),
    (Key::HOME, KeyCode::Home),
    (Key::END, KeyCode::End),
    (Key::PAGEUP, KeyCode::PageUp),
    (Key::PAGEDOWN, KeyCode::PageDown),
    (Key::BACKSPACE, KeyCode::Backspace),
    (Key::DELETE, KeyCode::Delete),
    (Key::ENTER, KeyCode::Enter),
    (Key::KP_ENTER, KeyCode::Enter),
    (Key::TAB, KeyCode::Tab),
    (Key::SPACE, KeyCode::Space),
    (Key::ESCAPE, KeyCode::Escape),
    (Key::MINUS, KeyCode::Minus),
    (Key::EQUAL, KeyCode::Equal),
    (Key::BRACKETLEFT, KeyCode::BracketLeft),
    (Key::BRACKETRIGHT, KeyCode::BracketRight),
    (Key::BACKSLASH, KeyCode::Backslash),
    (Key::SEMICOLON, KeyCode::Semicolon),
    (Key::APOSTROPHE, KeyCode::Quote),
    (Key::COMMA, KeyCode::Comma),
    (Key::PERIOD, KeyCode::Period),
    (Key::SLASH, KeyCode::Slash),
    (Key::QUOTELEFT, KeyCode::Backquote),
];

// Cranpose's form of a Godot key event: the physical key, the text it
// types, and the modifiers held.
pub(crate) fn key_event(key: &Gd<InputEventKey>) -> KeyEvent {
    let right = key.get_location() == KeyLocation::RIGHT;
    let code = match key.get_physical_keycode() {
        Key::SHIFT if right => KeyCode::ShiftRight,
        Key::SHIFT => KeyCode::ShiftLeft,
        Key::CTRL if right => KeyCode::ControlRight,
        Key::CTRL => KeyCode::ControlLeft,
        Key::ALT if right => KeyCode::AltRight,
        Key::ALT => KeyCode::AltLeft,
        Key::META if right => KeyCode::MetaRight,
        Key::META => KeyCode::MetaLeft,
        physical => KEYS
            .iter()
            .find(|(godot, _)| *godot == physical)
            .map_or(KeyCode::Unknown, |(_, code)| *code),
    };
    let (kind, text) = if key.is_pressed() {
        let text = char::from_u32(key.get_unicode())
            .filter(|typed| !typed.is_control())
            .map(String::from)
            .unwrap_or_default();
        (KeyEventType::KeyDown, text)
    } else {
        (KeyEventType::KeyUp, String::new())
    };
    let modifiers = Modifiers {
        shift: key.is_shift_pressed(),
        ctrl: key.is_ctrl_pressed(),
        alt: key.is_alt_pressed(),
        meta: key.is_meta_pressed(),
    };
    KeyEvent::new(code, text, modifiers, kind)
}
