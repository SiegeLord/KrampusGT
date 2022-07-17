use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct KeyCode(pub allegro::KeyCode);

impl KeyCode
{
	pub fn to_str(&self) -> &'static str
	{
		match self.0
		{
			allegro::KeyCode::A => "A",
			allegro::KeyCode::B => "B",
			allegro::KeyCode::C => "C",
			allegro::KeyCode::D => "D",
			allegro::KeyCode::E => "E",
			allegro::KeyCode::F => "F",
			allegro::KeyCode::G => "G",
			allegro::KeyCode::H => "H",
			allegro::KeyCode::I => "I",
			allegro::KeyCode::J => "J",
			allegro::KeyCode::K => "K",
			allegro::KeyCode::L => "L",
			allegro::KeyCode::M => "M",
			allegro::KeyCode::N => "N",
			allegro::KeyCode::O => "O",
			allegro::KeyCode::P => "P",
			allegro::KeyCode::Q => "Q",
			allegro::KeyCode::R => "R",
			allegro::KeyCode::S => "S",
			allegro::KeyCode::T => "T",
			allegro::KeyCode::U => "U",
			allegro::KeyCode::V => "V",
			allegro::KeyCode::W => "W",
			allegro::KeyCode::X => "X",
			allegro::KeyCode::Y => "Y",
			allegro::KeyCode::Z => "Z",
			allegro::KeyCode::_0 => "0",
			allegro::KeyCode::_1 => "1",
			allegro::KeyCode::_2 => "2",
			allegro::KeyCode::_3 => "3",
			allegro::KeyCode::_4 => "4",
			allegro::KeyCode::_5 => "5",
			allegro::KeyCode::_6 => "6",
			allegro::KeyCode::_7 => "7",
			allegro::KeyCode::_8 => "8",
			allegro::KeyCode::_9 => "9",
			allegro::KeyCode::Pad0 => "Pad0",
			allegro::KeyCode::Pad1 => "Pad1",
			allegro::KeyCode::Pad2 => "Pad2",
			allegro::KeyCode::Pad3 => "Pad3",
			allegro::KeyCode::Pad4 => "Pad4",
			allegro::KeyCode::Pad5 => "Pad5",
			allegro::KeyCode::Pad6 => "Pad6",
			allegro::KeyCode::Pad7 => "Pad7",
			allegro::KeyCode::Pad8 => "Pad8",
			allegro::KeyCode::Pad9 => "Pad9",
			allegro::KeyCode::F1 => "F1",
			allegro::KeyCode::F2 => "F2",
			allegro::KeyCode::F3 => "F3",
			allegro::KeyCode::F4 => "F4",
			allegro::KeyCode::F5 => "F5",
			allegro::KeyCode::F6 => "F6",
			allegro::KeyCode::F7 => "F7",
			allegro::KeyCode::F8 => "F8",
			allegro::KeyCode::F9 => "F9",
			allegro::KeyCode::F10 => "F10",
			allegro::KeyCode::F11 => "F11",
			allegro::KeyCode::F12 => "F12",
			allegro::KeyCode::Escape => "Escape",
			allegro::KeyCode::Tilde => "Tilde",
			allegro::KeyCode::Minus => "Minus",
			allegro::KeyCode::Equals => "Equals",
			allegro::KeyCode::Backspace => "Backspace",
			allegro::KeyCode::Tab => "Tab",
			allegro::KeyCode::Openbrace => "Openbrace",
			allegro::KeyCode::Closebrace => "Closebrace",
			allegro::KeyCode::Enter => "Enter",
			allegro::KeyCode::Semicolon => "Semicolon",
			allegro::KeyCode::Quote => "Quote",
			allegro::KeyCode::Backslash => "Backslash",
			allegro::KeyCode::Backslash2 => "Backslash2",
			allegro::KeyCode::Comma => "Comma",
			allegro::KeyCode::Fullstop => "Fullstop",
			allegro::KeyCode::Slash => "Slash",
			allegro::KeyCode::Space => "Space",
			allegro::KeyCode::Insert => "Insert",
			allegro::KeyCode::Delete => "Delete",
			allegro::KeyCode::Home => "Home",
			allegro::KeyCode::End => "End",
			allegro::KeyCode::PgUp => "PgUp",
			allegro::KeyCode::PgDn => "PgDn",
			allegro::KeyCode::Left => "Left",
			allegro::KeyCode::Right => "Right",
			allegro::KeyCode::Up => "Up",
			allegro::KeyCode::Down => "Down",
			allegro::KeyCode::PadSlash => "PadSlash",
			allegro::KeyCode::PadAsterisk => "PadAsterisk",
			allegro::KeyCode::PadMinus => "PadMinus",
			allegro::KeyCode::PadPlus => "PadPlus",
			allegro::KeyCode::PadDelete => "PadDelete",
			allegro::KeyCode::PadEnter => "PadEnter",
			allegro::KeyCode::PrintScreen => "PrintScreen",
			allegro::KeyCode::Pause => "Pause",
			allegro::KeyCode::AbntC1 => "AbntC1",
			allegro::KeyCode::Yen => "Yen",
			allegro::KeyCode::Kana => "Kana",
			allegro::KeyCode::Convert => "Convert",
			allegro::KeyCode::NoConvert => "NoConvert",
			allegro::KeyCode::At => "At",
			allegro::KeyCode::Circumflex => "Circumflex",
			allegro::KeyCode::Colon2 => "Colon2",
			allegro::KeyCode::Kanji => "Kanji",
			allegro::KeyCode::PadEquals => "PadEquals",
			allegro::KeyCode::Backquote => "Backquote",
			allegro::KeyCode::Semicolon2 => "Semicolon2",
			allegro::KeyCode::Command => "Command",
			allegro::KeyCode::Unknown => "Unknown",
			allegro::KeyCode::LShift => "LShift",
			allegro::KeyCode::RShift => "RShift",
			allegro::KeyCode::LCtrl => "LCtrl",
			allegro::KeyCode::RCtrl => "RCtrl",
			allegro::KeyCode::Alt => "Alt",
			allegro::KeyCode::AltGr => "AltGr",
			allegro::KeyCode::LWin => "LWin",
			allegro::KeyCode::RWin => "RWin",
			allegro::KeyCode::Menu => "Menu",
			allegro::KeyCode::ScrollLock => "ScrollLock",
			allegro::KeyCode::NumLock => "NumLock",
			allegro::KeyCode::CapsLock => "CapsLock",
		}
	}

	pub fn from_str(s: &str) -> Option<Self>
	{
		match s
		{
			"A" => Some(allegro::KeyCode::A),
			"B" => Some(allegro::KeyCode::B),
			"C" => Some(allegro::KeyCode::C),
			"D" => Some(allegro::KeyCode::D),
			"E" => Some(allegro::KeyCode::E),
			"F" => Some(allegro::KeyCode::F),
			"G" => Some(allegro::KeyCode::G),
			"H" => Some(allegro::KeyCode::H),
			"I" => Some(allegro::KeyCode::I),
			"J" => Some(allegro::KeyCode::J),
			"K" => Some(allegro::KeyCode::K),
			"L" => Some(allegro::KeyCode::L),
			"M" => Some(allegro::KeyCode::M),
			"N" => Some(allegro::KeyCode::N),
			"O" => Some(allegro::KeyCode::O),
			"P" => Some(allegro::KeyCode::P),
			"Q" => Some(allegro::KeyCode::Q),
			"R" => Some(allegro::KeyCode::R),
			"S" => Some(allegro::KeyCode::S),
			"T" => Some(allegro::KeyCode::T),
			"U" => Some(allegro::KeyCode::U),
			"V" => Some(allegro::KeyCode::V),
			"W" => Some(allegro::KeyCode::W),
			"X" => Some(allegro::KeyCode::X),
			"Y" => Some(allegro::KeyCode::Y),
			"Z" => Some(allegro::KeyCode::Z),
			"0" => Some(allegro::KeyCode::_0),
			"1" => Some(allegro::KeyCode::_1),
			"2" => Some(allegro::KeyCode::_2),
			"3" => Some(allegro::KeyCode::_3),
			"4" => Some(allegro::KeyCode::_4),
			"5" => Some(allegro::KeyCode::_5),
			"6" => Some(allegro::KeyCode::_6),
			"7" => Some(allegro::KeyCode::_7),
			"8" => Some(allegro::KeyCode::_8),
			"9" => Some(allegro::KeyCode::_9),
			"Pad0" => Some(allegro::KeyCode::Pad0),
			"Pad1" => Some(allegro::KeyCode::Pad1),
			"Pad2" => Some(allegro::KeyCode::Pad2),
			"Pad3" => Some(allegro::KeyCode::Pad3),
			"Pad4" => Some(allegro::KeyCode::Pad4),
			"Pad5" => Some(allegro::KeyCode::Pad5),
			"Pad6" => Some(allegro::KeyCode::Pad6),
			"Pad7" => Some(allegro::KeyCode::Pad7),
			"Pad8" => Some(allegro::KeyCode::Pad8),
			"Pad9" => Some(allegro::KeyCode::Pad9),
			"F1" => Some(allegro::KeyCode::F1),
			"F2" => Some(allegro::KeyCode::F2),
			"F3" => Some(allegro::KeyCode::F3),
			"F4" => Some(allegro::KeyCode::F4),
			"F5" => Some(allegro::KeyCode::F5),
			"F6" => Some(allegro::KeyCode::F6),
			"F7" => Some(allegro::KeyCode::F7),
			"F8" => Some(allegro::KeyCode::F8),
			"F9" => Some(allegro::KeyCode::F9),
			"F10" => Some(allegro::KeyCode::F10),
			"F11" => Some(allegro::KeyCode::F11),
			"F12" => Some(allegro::KeyCode::F12),
			"Escape" => Some(allegro::KeyCode::Escape),
			"Tilde" => Some(allegro::KeyCode::Tilde),
			"Minus" => Some(allegro::KeyCode::Minus),
			"Equals" => Some(allegro::KeyCode::Equals),
			"Backspace" => Some(allegro::KeyCode::Backspace),
			"Tab" => Some(allegro::KeyCode::Tab),
			"Openbrace" => Some(allegro::KeyCode::Openbrace),
			"Closebrace" => Some(allegro::KeyCode::Closebrace),
			"Enter" => Some(allegro::KeyCode::Enter),
			"Semicolon" => Some(allegro::KeyCode::Semicolon),
			"Quote" => Some(allegro::KeyCode::Quote),
			"Backslash" => Some(allegro::KeyCode::Backslash),
			"Backslash2" => Some(allegro::KeyCode::Backslash2),
			"Comma" => Some(allegro::KeyCode::Comma),
			"Fullstop" => Some(allegro::KeyCode::Fullstop),
			"Slash" => Some(allegro::KeyCode::Slash),
			"Space" => Some(allegro::KeyCode::Space),
			"Insert" => Some(allegro::KeyCode::Insert),
			"Delete" => Some(allegro::KeyCode::Delete),
			"Home" => Some(allegro::KeyCode::Home),
			"End" => Some(allegro::KeyCode::End),
			"PgUp" => Some(allegro::KeyCode::PgUp),
			"PgDn" => Some(allegro::KeyCode::PgDn),
			"Left" => Some(allegro::KeyCode::Left),
			"Right" => Some(allegro::KeyCode::Right),
			"Up" => Some(allegro::KeyCode::Up),
			"Down" => Some(allegro::KeyCode::Down),
			"PadSlash" => Some(allegro::KeyCode::PadSlash),
			"PadAsterisk" => Some(allegro::KeyCode::PadAsterisk),
			"PadMinus" => Some(allegro::KeyCode::PadMinus),
			"PadPlus" => Some(allegro::KeyCode::PadPlus),
			"PadDelete" => Some(allegro::KeyCode::PadDelete),
			"PadEnter" => Some(allegro::KeyCode::PadEnter),
			"PrintScreen" => Some(allegro::KeyCode::PrintScreen),
			"Pause" => Some(allegro::KeyCode::Pause),
			"AbntC1" => Some(allegro::KeyCode::AbntC1),
			"Yen" => Some(allegro::KeyCode::Yen),
			"Kana" => Some(allegro::KeyCode::Kana),
			"Convert" => Some(allegro::KeyCode::Convert),
			"NoConvert" => Some(allegro::KeyCode::NoConvert),
			"At" => Some(allegro::KeyCode::At),
			"Circumflex" => Some(allegro::KeyCode::Circumflex),
			"Colon2" => Some(allegro::KeyCode::Colon2),
			"Kanji" => Some(allegro::KeyCode::Kanji),
			"PadEquals" => Some(allegro::KeyCode::PadEquals),
			"Backquote" => Some(allegro::KeyCode::Backquote),
			"Semicolon2" => Some(allegro::KeyCode::Semicolon2),
			"Command" => Some(allegro::KeyCode::Command),
			"Unknown" => Some(allegro::KeyCode::Unknown),
			"LShift" => Some(allegro::KeyCode::LShift),
			"RShift" => Some(allegro::KeyCode::RShift),
			"LCtrl" => Some(allegro::KeyCode::LCtrl),
			"RCtrl" => Some(allegro::KeyCode::RCtrl),
			"Alt" => Some(allegro::KeyCode::Alt),
			"AltGr" => Some(allegro::KeyCode::AltGr),
			"LWin" => Some(allegro::KeyCode::LWin),
			"RWin" => Some(allegro::KeyCode::RWin),
			"Menu" => Some(allegro::KeyCode::Menu),
			"ScrollLock" => Some(allegro::KeyCode::ScrollLock),
			"NumLock" => Some(allegro::KeyCode::NumLock),
			"CapsLock" => Some(allegro::KeyCode::CapsLock),
			_ => None,
		}
		.map(Self)
	}
}

impl serde::Serialize for KeyCode
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.to_str())
	}
}

struct KeyCodeVisitor;

impl<'de> serde::de::Visitor<'de> for KeyCodeVisitor
{
	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
	{
		write!(formatter, "a KeyCode")
	}

	type Value = KeyCode;
	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		KeyCode::from_str(&value).ok_or(serde::de::Error::invalid_value(
			serde::de::Unexpected::Str(value),
			&self,
		))
	}
}

impl<'de> serde::Deserialize<'de> for KeyCode
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_str(KeyCodeVisitor)
	}
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Action
{
	TurnLeft,
	TurnRight,
	StrafeLeft,
	StrafeRight,
	MoveForward,
	MoveBackward,
	FireWeapon,
	SelectWeapon1,
	SelectWeapon2,
	SelectWeapon3,
	EnterVehicle,
}

impl Action
{
	pub fn to_str(&self) -> &'static str
	{
		match self
		{
			Action::TurnLeft => "TURN LEFT",
			Action::TurnRight => "TURN RIGHT",
			Action::StrafeLeft => "STRAFE LEFT",
			Action::StrafeRight => "STRAFE RIGHT",
			Action::MoveForward => "FORWARD",
			Action::MoveBackward => "BACKWARD",
			Action::FireWeapon => "FIRE WEAPON",
			Action::SelectWeapon1 => "WEAPON 1",
			Action::SelectWeapon2 => "WEAPON 2",
			Action::SelectWeapon3 => "WEAPON 3",
			Action::EnterVehicle => "ENTER VEHICLE",
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Controls
{
	pub controls: bimap::BiMap<Action, KeyCode>,
}

impl Controls
{
	pub fn new() -> Self
	{
		let mut controls = bimap::BiMap::new();
		controls.insert(Action::TurnLeft, KeyCode(allegro::KeyCode::Left));
		controls.insert(Action::TurnRight, KeyCode(allegro::KeyCode::Right));
		controls.insert(Action::StrafeLeft, KeyCode(allegro::KeyCode::A));
		controls.insert(Action::StrafeRight, KeyCode(allegro::KeyCode::D));
		controls.insert(Action::MoveForward, KeyCode(allegro::KeyCode::W));
		controls.insert(Action::MoveBackward, KeyCode(allegro::KeyCode::S));
		controls.insert(Action::FireWeapon, KeyCode(allegro::KeyCode::Space));
		controls.insert(Action::SelectWeapon1, KeyCode(allegro::KeyCode::_1));
		controls.insert(Action::SelectWeapon2, KeyCode(allegro::KeyCode::_2));
		controls.insert(Action::SelectWeapon3, KeyCode(allegro::KeyCode::_3));
		controls.insert(Action::EnterVehicle, KeyCode(allegro::KeyCode::E));
		Self { controls: controls }
	}

	pub fn decode_event(&self, event: &allegro::Event) -> Option<(bool, Action)>
	{
		match event
		{
			allegro::Event::KeyDown { keycode, .. } =>
			{
				if let Some(action) = self.controls.get_by_right(&KeyCode(*keycode))
				{
					return Some((true, *action));
				}
			}
			allegro::Event::KeyUp { keycode, .. } =>
			{
				if let Some(action) = self.controls.get_by_right(&KeyCode(*keycode))
				{
					return Some((false, *action));
				}
			}
			_ => (),
		}
		None
	}
}
