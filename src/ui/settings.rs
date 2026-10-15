use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub led_enabled: bool,
    pub wifi_ssid: String<32>,
}

#[derive(Deserialize, Debug)]
pub struct Action {
    pub id: u32,
    pub value: String<64>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Widget {
    #[serde(rename = "toggle")]
    Toggle {
        id: u32,
        label: String<32>,
        value: bool,
        parent_id: u32,
    },
    #[serde(rename = "slider")]
    Slider {
        id: u32,
        label: String<32>,
        min: u8,
        max: u8,
        value: u8,
    },
    #[serde(rename = "input")]
    Input {
        id: u32,
        label: String<32>,
        value: String<32>,
    },
    #[serde(rename = "button")]
    Button {
        id: u32,
        label: String<32>,
        value: String<32>,
        parent_id: u32,
    },

    #[serde(rename = "group")]
    Group {
        id: u32,
        parent: u32,
        label: String<32>,
        collapsed: bool,
    },
}

pub struct Builder {
    pub action: Option<Action>,
    next_id: u32,
    pub packet: Vec<Widget, 16>,
    pub changed: bool,
    parent_stack: Vec<u32, 8>,
}

impl Builder {
    pub fn new(action: Option<Action>) -> Self {
        Builder {
            action,
            next_id: 0,
            packet: Vec::new(),
            changed: false,
            parent_stack: Vec::new(),
        }
    }

    fn current_parent(&self) -> u32 {
        *self.parent_stack.last().unwrap_or(&0)
    }

    pub fn begin_group(&mut self, label: &str, collapsed: bool) -> u32 {
        let id = self.next_id();
        let parent = self.current_parent();
        let _ = self.packet.push(Widget::Group {
            id,
            parent,
            label: String::try_from(label).unwrap_or_default(),
            collapsed,
        });

        self.parent_stack.push(id).unwrap();

        id
    }

    pub fn end_group(&mut self) {
        self.parent_stack.pop();
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn switch(&mut self, label: &str, value: &mut bool) -> bool {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Toggle {
            id,
            label: String::try_from(label).unwrap_or_default(),
            value: *value,
            parent_id: self.current_parent(),
        });

        if let Some(ref act) = self.action {
            if act.id == id {
                if let Ok(v) = act.value.as_str().parse::<bool>() {
                    *value = v;
                    self.changed = true;
                    return true;
                }
            }
        }

        false
    }
}

pub fn build(b: &mut Builder, settings: &mut Settings) {
    b.begin_group("Buttons", false);
    b.switch("LED", &mut settings.led_enabled);
    b.switch("TOOL", &mut settings.led_enabled);
    b.end_group();

    b.begin_group("Controls", false);
    b.switch("LED1", &mut settings.led_enabled);
    b.switch("TOOL1", &mut settings.led_enabled);
    b.end_group();
}
