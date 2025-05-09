use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use xmltree::{Element as XMLElement, XMLNode};
use iced::{Element, Length, Subscription, Task};
use iced::widget::{button, container, svg, text_input, Column, Row, Scrollable, Text};
use crate::dependency::build_dependency_graph;
use iced::futures::stream;

use tokio::sync::watch;
use mermaid_rs::Mermaid;

#[derive(Debug, Clone)]
pub enum Message {
    UpdateInputVal(String),
    AskDependency,
    DependencyReceived(Result<(), String>),
    ProjectDependenciesUpdated,
    ImageGenerated(svg::Handle),
}

#[derive(Clone)]
pub struct AppState {
    project_dependencies: Arc<RwLock<Vec<(String, String)>>>,
    input_value: String,
    notifier: watch::Sender<()>,
    handle: Option<iced::widget::svg::Handle>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { 
            project_dependencies: Default::default(), 
            input_value: Default::default(), 
            notifier: watch::channel(()).0,
            handle: None,
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
        
        let mut top_row = Row::new().spacing(5).padding(8);
        top_row = top_row.push(text_input("Enter project path...", &self.input_value).on_input(|x| Message::UpdateInputVal(x)));
        top_row = top_row.push(
            match self.input_value.is_empty() {
                true => button("Analyze"),
                false => button("Analyze").on_press(Message::AskDependency),
            }
        );

        for (to, into) in self.project_dependencies.read().unwrap().clone() {
            let s = format!("{to} -> {into}");
            deps_column = deps_column.push(Text::new(s));
        }

        let displayed_image = match &self.handle {
            Some(handle) => {
                iced::widget::svg(handle.clone())
            }
            None => {
                iced::widget::svg("")
            }
        };

        let scroll = Scrollable::new(Column::new().push(deps_column).push(displayed_image));
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
                self.handle = None;

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
            Message::DependencyReceived(_res) => {
                let deps_borr = self.project_dependencies.clone();
                Task::perform(image_generation(deps_borr), Message::ImageGenerated)
            },
            Message::ImageGenerated(res) => {
                self.handle = Some(res);
                // This is where you would update the image in the UI
                // For now, we just return none
                Task::none()
            }
            Message::ProjectDependenciesUpdated => Task::none()
        }
    }
}

fn process_element(elem: &mut XMLElement) {
    elem.children.iter_mut().for_each(|child| {
        if let XMLNode::Element(child_elem) = child {
            // If it's a foreignObject
            if child_elem.name == "foreignObject" {
                // Try to extract the <p> text
                if let Some(XMLNode::Element(div)) = child_elem.children.get(0) {
                    if let Some(XMLNode::Element(span)) = div.children.get(0) {
                        if let Some(XMLNode::Element(p)) = span.children.get(0) {
                            if let Some(XMLNode::Text(text_content)) = p.children.get(0) {
                                // Replace the foreignObject with a <text> node
                                let mut new_text = XMLElement::new("text");
                                new_text.attributes.insert("x".into(), "30".into()); // Default (you can calculate better positions if needed)
                                new_text.attributes.insert("y".into(), "0".into());
                                new_text.attributes.insert("font-size".into(), "16".into());
                                new_text.attributes.insert("text-anchor".into(), "middle".into());
                                new_text.attributes.insert("dominant-baseline".into(), "middle".into());
                                new_text.attributes.insert("fill".into(), "#333".into());
                                new_text.children.push(XMLNode::Text(text_content.clone()));

                                *child_elem = new_text; // Overwrite
                            }
                        }
                    }
                }
            } else {
                // Recursive: process all child elements
                process_element(child_elem);
            }
        }
    });
}

async fn image_generation(project_dependencies: Arc<RwLock<Vec<(String, String)>>>) -> iced::widget::svg::Handle {
    let mermaid = Mermaid::new().unwrap();
    let mut graph = String::from("graph TD\n");
    for el in project_dependencies.read().unwrap().iter() {
        graph.push_str(&format!("{} --> {}\n", el.0, el.1));
    }
    let svg = mermaid.render(&graph).unwrap();
    let mut root = XMLElement::parse(svg.as_bytes()).unwrap();
    process_element(&mut root);
    let mut output = Vec::new();
    root.write(&mut output).unwrap();
    iced::widget::svg::Handle::from_memory(output)
}