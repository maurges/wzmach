use serde::Deserialize;
use uinput::event::keyboard::Key;

#[derive(PartialEq, Debug)]
pub struct ConfigKey(pub Key);

impl<'de> Deserialize<'de> for ConfigKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor)
    }
}

struct KeyVisitor;
impl<'de> serde::de::Visitor<'de> for KeyVisitor {
    type Value = ConfigKey;

    fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "Expecting Key string")
    }

    fn visit_str<E: serde::de::Error>(self, data: &str) -> Result<ConfigKey, E> {
        static VARIANTS: &[&str] = &[&"KEY"];
        match KEYS_TABLE.get(data) {
            Some(k) => Ok(ConfigKey(*k)),
            None => Err(E::unknown_variant(data, VARIANTS)),
        }
    }
}

// copy-pasted from uinput sources
const KEYS_TABLE: phf::Map<&'static str, Key> = phf::phf_map! {
    "Esc" => Key::Esc,
    "1" => Key::_1,
    "2" => Key::_2,
    "3" => Key::_3,
    "4" => Key::_4,
    "5" => Key::_5,
    "6" => Key::_6,
    "7" => Key::_7,
    "8" => Key::_8,
    "9" => Key::_9,
    "0" => Key::_0,
    "Minus" => Key::Minus,
    "Equal" => Key::Equal,
    "BackSpace" => Key::BackSpace,
    "Tab" => Key::Tab,
    "Q" => Key::Q,
    "W" => Key::W,
    "E" => Key::E,
    "R" => Key::R,
    "T" => Key::T,
    "Y" => Key::Y,
    "U" => Key::U,
    "I" => Key::I,
    "O" => Key::O,
    "P" => Key::P,
    "LeftBrace" => Key::LeftBrace,
    "RightBrace" => Key::RightBrace,
    "Enter" => Key::Enter,
    "LeftControl" => Key::LeftControl,
    "A" => Key::A,
    "S" => Key::S,
    "D" => Key::D,
    "F" => Key::F,
    "G" => Key::G,
    "H" => Key::H,
    "J" => Key::J,
    "K" => Key::K,
    "L" => Key::L,
    "SemiColon" => Key::SemiColon,
    "Apostrophe" => Key::Apostrophe,
    "Grave" => Key::Grave,
    "LeftShift" => Key::LeftShift,
    "BackSlash" => Key::BackSlash,
    "Z" => Key::Z,
    "X" => Key::X,
    "C" => Key::C,
    "V" => Key::V,
    "B" => Key::B,
    "N" => Key::N,
    "M" => Key::M,
    "Comma" => Key::Comma,
    "Dot" => Key::Dot,
    "Slash" => Key::Slash,
    "RightShift" => Key::RightShift,
    "LeftAlt" => Key::LeftAlt,
    "Space" => Key::Space,
    "CapsLock" => Key::CapsLock,
    "F1" => Key::F1,
    "F2" => Key::F2,
    "F3" => Key::F3,
    "F4" => Key::F4,
    "F5" => Key::F5,
    "F6" => Key::F6,
    "F7" => Key::F7,
    "F8" => Key::F8,
    "F9" => Key::F9,
    "F10" => Key::F10,
    "NumLock" => Key::NumLock,
    "ScrollLock" => Key::ScrollLock,
    "F11" => Key::F11,
    "F12" => Key::F12,
    "RightControl" => Key::RightControl,
    "SysRq" => Key::SysRq,
    "RightAlt" => Key::RightAlt,
    "LineFeed" => Key::LineFeed,
    "Home" => Key::Home,
    "Up" => Key::Up,
    "PageUp" => Key::PageUp,
    "Left" => Key::Left,
    "Right" => Key::Right,
    "End" => Key::End,
    "Down" => Key::Down,
    "PageDown" => Key::PageDown,
    "Insert" => Key::Insert,
    "Delete" => Key::Delete,
    "LeftMeta" => Key::LeftMeta,
    "RightMeta" => Key::RightMeta,
    "ScrollUp" => Key::ScrollUp,
    "ScrollDown" => Key::ScrollDown,
    "F13" => Key::F13,
    "F14" => Key::F14,
    "F15" => Key::F15,
    "F16" => Key::F16,
    "F17" => Key::F17,
    "F18" => Key::F18,
    "F19" => Key::F19,
    "F20" => Key::F20,
    "F21" => Key::F21,
    "F22" => Key::F22,
    "F23" => Key::F23,
    "F24" => Key::F24,
};
