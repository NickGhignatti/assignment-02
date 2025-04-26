use std::fmt::{format, write, Display, Formatter};

#[derive(Clone)]
pub struct ClassDepsReport {
    pub class_name: String,
    pub class_deps: Vec<String>,
    pub nested_classes: Vec<ClassDepsReport>
}

impl Display for ClassDepsReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut report: String = String::new();

        report.push_str(format!("========{}========\n", self.class_name).as_str());

        let class_dependencies = "dependencies: \n";
        report.push_str(class_dependencies);
        for dep in self.class_deps.clone() {
            report.push_str(format!("{} \n", dep).as_str())
        }

        let nested_classes = "nested classes: \n";
        report.push_str(nested_classes);
        for nested_class in self.nested_classes.clone() {
            report.push_str(format!("{} \n", nested_class.to_string()).as_str())
        }

        report.push_str(format!("========{}========", self.class_name).as_str());

        write!(f, "{}", report)
    }
}