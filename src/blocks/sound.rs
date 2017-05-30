use std::time::Duration;
use std::process::Command;

use block::Block;
use widgets::button::ButtonWidget;
use widget::{I3BarWidget, State};
use input::I3barEvent;

use serde_json::Value;
use uuid::Uuid;


// TODO: Use the alsa control bindings to implement push updates
// TODO: Allow for custom audio devices instead of Master
pub struct Sound {
    text: ButtonWidget,
    id: String,
    update_interval: Duration,
    theme: Value,
}

impl Sound {
    pub fn new(config: Value, theme: Value) -> Sound {
        {
            let id = Uuid::new_v4().simple().to_string();
            Sound {
                update_interval: Duration::new(get_u64_default!(config, "interval", 2), 0),
                text: ButtonWidget::new(theme.clone(), &id).with_icon("volume_empty"),
                id,
                theme,
            }
        }
    }
}

// To filter [100%] output from amixer into 100
const FILTER: &[char] = &['[', ']', '%'];

impl Block for Sound {
    fn update(&mut self) -> Option<Duration> {
        if let Some((volume, muted)) = get_audio_details() {
            if muted {
                self.text.set_icon("volume_empty");
                self.text
                    .set_text(self.theme["icons"]["volume_muted"]
                                  .as_str()
                                  .expect("Wrong icon identifier!")
                                  .to_owned());
                self.text.set_state(State::Warning);
            } else {
                self.text
                    .set_icon(match volume {
                                  0...20 => "volume_empty",
                                  20...70 => "volume_half",
                                  _ => "volume_full",
                              });
                self.text.set_text(format!("{:02}%", volume));
                self.text.set_state(State::Info);
            }
        } else {
            // TODO: Do proper error handling here instead of hiding in a corner
            self.text.set_icon("");
            self.text.set_text("".to_owned());
            self.text.set_state(State::Idle);
        }

        Some(self.update_interval.clone())
    }
    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.text]
    }
    fn click(&mut self, e: &I3barEvent) {
        if let Some(ref name) = e.name {
            if name.as_str() == self.id {
                if let Some((_, muted)) = get_audio_details() {
                    match muted {
                        true => {
                            //is already muted so unmute it
                            Command::new("amixer")
                                .args(&["set", "Master", "on"])
                                .output()
                                .ok();
                            Command::new("amixer")
                                .args(&["set", "Headphone", "on"])
                                .output()
                                .ok();
                            Command::new("amixer")
                                .args(&["set", "Front", "on"])
                                .output()
                                .ok();
                            self.update();
                        }
                        false => {
                            //is unmuted so mute it
                            Command::new("amixer")
                                .args(&["set", "Master", "off"])
                                .output()
                                .ok();
                            self.update();
                        }
                    }
                }
            }
        }
    }
    fn id(&self) -> &str {
        &self.id
    }
}

fn get_audio_details() -> Option<(u64, bool)> {
    let output = Command::new("amixer")
        .args(&["get", "Master"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());

    if let Ok(output) = output {
        let last = (&output)
            .lines()
            .into_iter()
            .last()
            .unwrap()
            .split_whitespace()
            .into_iter()
            .filter(|x| x.starts_with('[') && !x.contains("dB"))
            .map(|s| s.trim_matches(FILTER))
            .collect::<Vec<&str>>();

        Some((last[0].parse::<u64>().unwrap(),
              match last[1] {
                  "on" => false,
                  "off" => true,
                  _ => false,
              }))
    } else {
        None
    }
}
