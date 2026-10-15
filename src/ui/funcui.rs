use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use defmt::{error};

pub static REDRAW_UI: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub const PARENT_STACK_SIZE: usize = 100;
pub const MAX_SELECT_OPTIONS: usize = 20;

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
        parent_id: u32,
        label: String<32>,
        value: bool,
    },
    #[serde(rename = "indicator")]
    Indicator {
        id: u32,
        parent_id: u32,
        label: String<32>,
        enabled: bool,
    },
    #[serde(rename = "slider")]
    Slider {
        id: u32,
        parent_id: u32,
        label: String<32>,
        min: u8,
        max: u8,
        value: u8,
    },
    #[serde(rename = "input")]
    Input {
        id: u32,
        parent_id: u32,
        label: String<32>,
        placeholder: String<32>,
        value: String<32>,
        input_type: InputType,
    },
    #[serde(rename = "button")]
    Button {
        id: u32,
        parent_id: u32,
        label: String<32>,
        value: String<32>,
    },
    #[serde(rename = "label")]
    Label {
        id: u32,
        parent_id: u32,
        label: String<32>,
        text: String<32>,
    },
    #[serde(rename = "progressbar")]
    ProgressBar {
        id: u32,
        parent_id: u32,
        label: String<32>,
        text: String<64>,
        progress: u8,
    },
    #[serde(rename = "progress_ring")]
    ProgressRing {
        id: u32,
        parent_id: u32,
        label: String<32>,
        in_progress: bool,
    },
    #[serde(rename = "select")]
    Select {
        id: u32,
        parent: u32,
        label: String<32>,
        selected_index: u8,
        options: heapless::Vec<String<32>, MAX_SELECT_OPTIONS>,
    },
    #[serde(rename = "log")]
    Log {
        id: u32,
        parent_id: u32,
        label: String<32>,
        text: String<64>,
    },
    #[serde(rename = "notification")]
    Notification {
        id: u32,
        parent_id: u32,
        kind: NotificationKind,
        message: String<32>,
    },
    #[serde(rename = "group")]
    Group {
        id: u32,
        parent: u32,
        label: String<32>,
        collapsed: bool,
        alignment: Alignment,
    },
    #[serde(rename = "tab")]
    Tab {
        id: u32,
        parent: u32,
        label: String<32>,
        collapsed: bool,
    },
}

#[derive(Serialize)]
#[serde(tag = "alignment")]
pub enum Alignment {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
}

#[derive(Serialize)]
#[serde(tag = "group_widget_kind")]
enum GroupWidgetKind {
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "tab")]
    Tab,
}

#[derive(Serialize)]
#[serde(tag = "notification_kind")]
pub enum NotificationKind {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize)]
pub enum InputType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "email")]
    Email,
}

impl Default for InputType {
    fn default() -> Self {
        InputType::Text
    }
}

pub struct Builder {
    pub action: Option<Action>,
    next_id: u32,
    pub packet: Vec<Widget, PARENT_STACK_SIZE>,
    pub changed: bool,
    parent_stack: Vec<u32, PARENT_STACK_SIZE>,
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

    pub fn tab<F>(&mut self, label: &str, selected: bool, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.section(
            GroupWidgetKind::Tab,
            label,
            selected,
            Alignment::Horizontal,
            f,
        )
    }

    pub fn group<F>(
        &mut self,
        label: &str,
        collapsed: bool,
        alignment: Alignment,
        f: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.section(GroupWidgetKind::Group, label, collapsed, alignment, f)
    }

    fn section<F>(
        &mut self,
        widget: GroupWidgetKind,
        label: &str,
        collapsed: bool,
        alignment: Alignment,
        f: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let id = self.next_id();
        let parent = self.current_parent();

        let w = match widget {
            GroupWidgetKind::Group => Widget::Group {
                id,
                parent,
                label: String::try_from(label).unwrap_or_default(),
                collapsed,
                alignment,
            },
            GroupWidgetKind::Tab => Widget::Tab {
                id,
                parent,
                label: String::try_from(label).unwrap_or_default(),
                collapsed,
            },
        };

        let _ = self.packet.push(w);

        // Push to stack
        if self.parent_stack.push(id).is_err() {
           error!("Parent stack overflow, on id {}", id);
        }

        // Execute the nested builder calls
        f(self);

        // Pop to close-end group
        self.parent_stack.pop();

        self
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

        if let Some(ref act) = self.action
            && act.id == id
            && let Ok(v) = act.value.as_str().parse::<bool>()
        {
            *value = v;
            self.changed = true;
            return true;
        }

        false
    }

    pub fn button(&mut self, label: &str, value: &str) -> bool {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Button {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap_or_default(),
            value: String::try_from(value).unwrap_or_default(),
        });

        if let Some(ref act) = self.action
            && act.id == id
        {
            self.changed = true;
            return true;
        }

        false
    }

    pub fn slider(&mut self, label: &str, min: u8, max: u8, value: &mut u8) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Slider {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap_or_default(),
            min,
            max,
            value: *value,
        });

        if let Some(ref action) = self.action
            && action.id == id
            && let Ok(v) = action.value.as_str().parse::<u8>()
        {
            *value = v;
            self.changed = true;
        }

        self
    }

    pub fn input(
        &mut self,
        label: &str,
        placeholder: &str,
        input_type: InputType,
        value: &mut String<32>,
    ) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Input {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap_or_default(),
            value: String::try_from(value.as_str()).unwrap_or_default(),
            placeholder: String::try_from(placeholder).unwrap_or_default(),
            input_type,
        });

        if let Some(ref action) = self.action
            && action.id == id
        {
            value.clear();
            value.push_str(&action.value.as_str()).unwrap();
            self.changed = true;
        }

        self
    }

    pub fn select(&mut self, label: &str, options: &[&str], selected_index: &mut u8) -> &mut Self {
        let id = self.next_id();
        let mut opts: Vec<String<32>, MAX_SELECT_OPTIONS> = Vec::new();
        for &opt in options.iter().take(MAX_SELECT_OPTIONS) {
            opts.push(String::try_from(opt).unwrap_or_default())
                .unwrap();
        }

        if selected_index >= opts.len().try_into().as_mut().unwrap() {
            // ERROR
        }

        let _ = self.packet.push(Widget::Select {
            id,
            parent: self.current_parent(),
            label: String::try_from(label).unwrap(),
            selected_index: *selected_index,
            options: opts,
        });

        if let Some(ref act) = self.action {
            if act.id == id {
                let raw = act.value.as_str();

                match raw.parse::<u8>() {
                    Ok(new_selected_index) => {
                        *selected_index = new_selected_index;
                        self.changed = true;
                    }
                    Err(_) => todo!(),
                }
            }
        }

        self
    }

    pub fn indicator(&mut self, label: &str, enabled: &mut bool) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Indicator {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap(),
            enabled: *enabled,
        });

        if let Some(ref act) = self.action {
            if act.id == id
                && let Ok(v) = act.value.as_str().parse::<bool>()
            {
                *enabled = v;
            }
        }

        self
    }

    pub fn progress_bar(&mut self, label: &str, text: &str, progress: &mut u8) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::ProgressBar {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap(),
            text: String::try_from(text).unwrap(),
            progress: *progress,
        });

        if let Some(ref act) = self.action {
            if act.id == id
                && let Ok(v) = act.value.as_str().parse::<u8>()
            {
                *progress = v;
            }
        }

        self
    }

    pub fn progress_ring(&mut self, label: &str, in_progress: &mut bool) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::ProgressRing {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap(),
            in_progress: *in_progress,
        });

        if let Some(ref act) = self.action {
            if act.id == id
                && let Ok(v) = act.value.as_str().parse::<bool>()
            {
                *in_progress = v;
            }
        }

        self
    }

    pub fn log(&mut self, label: &str, log: &str) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Log {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap(),
            text: String::try_from(log).unwrap(),
        });

        self
    }

    pub fn label(&mut self, label: &str, text: &mut String<32>) -> &mut Self {
        let id = self.next_id();

        let _ = self.packet.push(Widget::Label {
            id,
            parent_id: self.current_parent(),
            label: String::try_from(label).unwrap(),
            text: String::try_from(text.as_str()).unwrap(),
        });

        if let Some(ref action) = self.action
            && action.id == id
        {
            text.clear();
            text.push_str(&action.value.as_str()).unwrap();
            self.changed = true;
        }

        self
    }
}
