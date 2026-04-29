use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, InputEvent, KeyCode};
use std::thread;
use std::time::Duration;

const EV_KEY: u16 = 0x01;

const KEY_LEFTCTRL: u16 = 29;
const KEY_C: u16 = 46;
const KEY_V: u16 = 47;

const SUPPORTED_KEYS: &[u16] = &[
    KEY_LEFTCTRL, KEY_C, KEY_V,
    13,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 43,
    44, 45, 46, 47, 48, 49, 50, 51, 52,
];

pub struct UinputController {
    device: VirtualDevice,
}

impl UinputController {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for &keycode in SUPPORTED_KEYS {
            keys.insert(KeyCode(keycode));
        }

        let device = VirtualDevice::builder()?
            .name("keycast virtual keyboard")
            .with_keys(&keys)?
            .build()?;

        Ok(Self { device })
    }

    #[inline]
    fn emit(&mut self, events: &[InputEvent]) {
        if let Err(err) = self.device.emit(events) {
            eprintln!("uinput emit error: {err}");
        }
    }

    pub fn simulate_ctrl_c(&mut self) {
        self.emit(&[
            InputEvent::new(EV_KEY, KEY_LEFTCTRL, 1),
            InputEvent::new(EV_KEY, KEY_C, 1),
        ]);
        thread::sleep(Duration::from_millis(1));
        self.emit(&[
            InputEvent::new(EV_KEY, KEY_C, 0),
            InputEvent::new(EV_KEY, KEY_LEFTCTRL, 0),
        ]);
        thread::sleep(Duration::from_millis(30));
    }

    pub fn simulate_ctrl_v(&mut self) {
        self.emit(&[
            InputEvent::new(EV_KEY, KEY_LEFTCTRL, 1),
            InputEvent::new(EV_KEY, KEY_V, 1),
        ]);
        thread::sleep(Duration::from_millis(1));
        self.emit(&[
            InputEvent::new(EV_KEY, KEY_V, 0),
            InputEvent::new(EV_KEY, KEY_LEFTCTRL, 0),
        ]);
    }
}
