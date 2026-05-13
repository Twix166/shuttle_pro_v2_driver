use crate::config::CompiledProfile;
use crate::input::{InputEvent, ABS_MISC, EV_ABS, EV_KEY, EV_REL, REL_DIAL};
use crate::keys::KeyChord;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapperAction {
    Chords(Vec<KeyChord>),
    StartRepeat {
        chords: Vec<KeyChord>,
        interval: Duration,
    },
    StopRepeat,
}

pub struct Mapper {
    profile: CompiledProfile,
    last_shuttle: i32,
}

impl Mapper {
    pub fn new(profile: CompiledProfile) -> Self {
        Self {
            profile,
            last_shuttle: 0,
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> Vec<MapperAction> {
        match event.event_type {
            EV_KEY => self.handle_button(event.code, event.value),
            EV_REL if event.code == REL_DIAL => self.handle_jog(event.value),
            EV_ABS if event.code == ABS_MISC => self.handle_shuttle(event.value),
            _ => Vec::new(),
        }
    }

    fn handle_button(&self, code: u16, value: i32) -> Vec<MapperAction> {
        if code < self.profile.device.button_base {
            return Vec::new();
        }

        let number = (code - self.profile.device.button_base + 1) as u8;
        let Some(button) = self.profile.buttons.get(&number) else {
            return Vec::new();
        };

        match value {
            1 => vec![MapperAction::Chords(button.press.clone())],
            0 => vec![MapperAction::Chords(button.release.clone())],
            _ => Vec::new(),
        }
    }

    fn handle_jog(&self, value: i32) -> Vec<MapperAction> {
        if value > 0 {
            vec![MapperAction::Chords(self.profile.jog.positive.clone())]
        } else if value < 0 {
            vec![MapperAction::Chords(self.profile.jog.negative.clone())]
        } else {
            Vec::new()
        }
    }

    fn handle_shuttle(&mut self, value: i32) -> Vec<MapperAction> {
        let value = value.clamp(-7, 7);

        if value == self.last_shuttle {
            return Vec::new();
        }

        self.last_shuttle = value;

        if value == 0 {
            let mut actions = vec![MapperAction::StopRepeat];
            if !self.profile.shuttle.neutral.is_empty() {
                actions.push(MapperAction::Chords(self.profile.shuttle.neutral.clone()));
            }
            return actions;
        }

        let level = value.unsigned_abs() as usize;
        let rate = self.profile.shuttle.rates_per_second[level].max(1);
        let interval = Duration::from_millis(1000 / u64::from(rate));
        let chords = if value > 0 {
            self.profile.shuttle.positive.clone()
        } else {
            self.profile.shuttle.negative.clone()
        };

        vec![MapperAction::StartRepeat { chords, interval }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    fn profile() -> CompiledProfile {
        toml::from_str::<Profile>(
            r#"
[profile]
name = "test"

[jog]
positive = ["right"]
negative = ["left"]

[shuttle]
positive = ["l"]
negative = ["j"]
neutral = ["k"]
rates_per_second = [0, 2, 4, 6, 8, 10, 12, 14]

[buttons.1]
press = ["space"]
"#,
        )
        .unwrap()
        .compile()
        .unwrap()
    }

    #[test]
    fn maps_button_press() {
        let mut mapper = Mapper::new(profile());
        let actions = mapper.handle_event(event(EV_KEY, 704, 1));

        assert!(
            matches!(&actions[0], MapperAction::Chords(chords) if chords[0].key.name == "space")
        );
    }

    #[test]
    fn maps_jog_direction() {
        let mut mapper = Mapper::new(profile());
        let actions = mapper.handle_event(event(EV_REL, REL_DIAL, -1));

        assert!(
            matches!(&actions[0], MapperAction::Chords(chords) if chords[0].key.name == "left")
        );
    }

    #[test]
    fn starts_and_stops_shuttle_repeat() {
        let mut mapper = Mapper::new(profile());

        assert!(matches!(
            &mapper.handle_event(event(EV_ABS, ABS_MISC, 3))[0],
            MapperAction::StartRepeat { .. }
        ));
        assert!(matches!(
            &mapper.handle_event(event(EV_ABS, ABS_MISC, 0))[0],
            MapperAction::StopRepeat
        ));
    }

    fn event(event_type: u16, code: u16, value: i32) -> InputEvent {
        InputEvent {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            event_type,
            code,
            value,
        }
    }
}
