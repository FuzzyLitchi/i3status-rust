use widget::State;
use serde_json::Value;
use super::super::widget::I3BarWidget;

#[derive(Clone,Debug)]
pub struct ButtonWidget {
    content: Option<String>,
    icon: Option<String>,
    state: State,
    id: String,
    rendered: Value,
    cached_output: Option<String>,
    theme: Value,
}

impl ButtonWidget {
    pub fn new(theme: Value, id: &str) -> Self {
        ButtonWidget {
            content: None,
            icon: None,
            state: State::Idle,
            id: String::from(id),
            rendered: json!({
                "full_text": "",
                "separator": false,
                "separator_block_width": 0,
                "background": "#000000",
                "color": "#000000"
            }),
            theme: theme,
            cached_output: None,
        }
    }

    pub fn with_icon(mut self, name: &str) -> Self {
        self.icon = Some(String::from(self.theme["icons"][name]
                                          .as_str()
                                          .expect("Wrong icon identifier!")));
        self.update();
        self
    }

    pub fn with_text(mut self, content: &str) -> Self {
        self.content = Some(String::from(content));
        self.update();
        self
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.state = state;
        self.update();
        self
    }

    pub fn set_text(&mut self, content: String) {
        self.content = Some(content);
        self.update();
    }

    pub fn set_icon(&mut self, name: &str) {
        self.icon = Some(String::from(self.theme["icons"][name]
                                          .as_str()
                                          .expect("Wrong icon identifier!")));
        self.update();
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        self.update();
    }

    fn update(&mut self) {
        let (key_bg, key_fg) = self.state.theme_keys();

        self.rendered = json!({
            "full_text": format!("{}{} ",
                                self.icon.clone().unwrap_or(String::from(" ")),
                                self.content.clone().unwrap_or(String::from(""))),
            "separator": false,
            "name": self.id.clone(),
            "separator_block_width": 0,
            "background": self.theme[key_bg],
            "color": self.theme[key_fg]
        });

        self.cached_output = Some(self.rendered.to_string());
    }
}

impl I3BarWidget for ButtonWidget {
    fn to_string(&self) -> String {
        self.cached_output
            .clone()
            .unwrap_or(self.rendered.to_string())
    }

    fn get_rendered(&self) -> &Value {
        &self.rendered
    }
}
