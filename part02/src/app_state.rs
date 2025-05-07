use iced::Element;
use iced::widget::{button, text_input, Column, Row, Text};

#[derive(Debug, Clone)]
pub enum Message {
    GetDependency,
    AskDependency,
    InputChanged,
    AnalyzePressed,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    project_dependencies: Vec<(String, String)>,
    input_value: String
}

impl AppState {
    pub fn view<'a>(&self) -> Element<'_, Message> {
        let mut view_struct = Column::new();

        // 1) Top row: input + button
        let mut top_row = Row::new().spacing(5).padding(8);
        top_row = top_row.push(text_input("Enter project path...", &self.input_value));
        top_row = top_row.push(button("Analyze").on_press(Message::AskDependency));

        view_struct = view_struct.push(top_row);

        // 2) Scrollable list of dependencies
        let mut deps_column = Column::new().spacing(5).padding(10);

        for (to, into) in self.project_dependencies.clone() {
            let s = format!("{to} -> {into}");
            deps_column = deps_column.push(Text::new(s));
        }

        view_struct = view_struct.push(deps_column);

        view_struct.into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::GetDependency => {}
            Message::AskDependency => {
                for _ in 0..10 {
                    self.project_dependencies.push(("A".to_string(), "B".to_string()));
                }
            }
            _ => {}
        }
    }
}