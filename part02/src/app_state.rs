use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use iced::futures::stream;
use iced::{Element, Length, Subscription, Task};
use iced::widget::{button, container, text_input, Column, Row, Scrollable, Text};
use crate::dependency::build_dependency_graph;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub enum Message {
    UpdateInputVal(String),
    AskDependency,
    DependencyReceived(Result<(), String>),
    ProjectDependenciesUpdated,
}

#[derive(Clone)]
pub struct AppState {
    project_dependencies: Arc<RwLock<Vec<(String, String)>>>,
    input_value: String,
    notifier: watch::Sender<()>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { 
            project_dependencies: Default::default(), 
            input_value: Default::default(), 
            notifier: watch::channel(()).0,
        }
    }
}

impl AppState {

    pub fn subscription(&self) -> Subscription<Message> {
        let receiver = self.notifier.subscribe();
    
        Subscription::run_with_id(
            (),
            (move || {
                stream::unfold(receiver, |mut receiver| async move {
                    match receiver.changed().await {
                        Ok(_) => Some((Message::ProjectDependenciesUpdated, receiver)),
                        Err(_) => None
                    }
                })
            })()
        )
    }

    pub fn view<'a>(&self) -> Element<'_, Message> {
        let mut deps_column = Column::new().spacing(5).padding(10);

        // 1) Top row: input + button
        let mut top_row = Row::new().spacing(5).padding(8);
        top_row = top_row.push(text_input("Enter project path...", &self.input_value).on_input(|x| Message::UpdateInputVal(x)));
        top_row = top_row.push(
            match self.input_value.is_empty() {
                true => button("Analyze"),
                false => button("Analyze").on_press(Message::AskDependency),
            }
        );
        // 2) Scrollable list of dependencies

        for (to, into) in self.project_dependencies.read().unwrap().clone() {
            let s = format!("{to} -> {into}");
            deps_column = deps_column.push(Text::new(s));
        }

        let scroll = Scrollable::new(deps_column)
            .width(Length::Fill)
            .height(Length::Fill);

        container(Column::new().push(top_row).push(scroll).spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateInputVal(x) => {
                self.input_value = x;
                Task::none()
            }
            Message::AskDependency => {
                self.project_dependencies.write().unwrap().clear();

                let path = PathBuf::from(self.input_value.clone());
                if !path.exists() {
                    return Task::none();
                }
                
                let deps_borr = self.project_dependencies.clone();
                let notifier_borr = self.notifier.clone();

                Task::perform(async move {
                    build_dependency_graph(path.clone(), deps_borr, notifier_borr).await
                }, Message::DependencyReceived)
            }
            Message::DependencyReceived(_res) => Task::none(),
            Message::ProjectDependenciesUpdated => Task::none()
        }
    }
}