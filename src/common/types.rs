use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct ClassDepsReport {
    pub class_name: String,
    pub class_deps: Vec<String>,
    pub nested_classes: Vec<ClassDepsReport>
}

fn get_string_with_nesting_level(class: ClassDepsReport, nes_level: i8) -> String {
    let mut tab = String::new();

    for _ in 0..nes_level {
        tab.push_str("|    ");
    }

    let mut report = String::new();
    report.push_str(format!("{tab}|{}\n", class.class_name).as_str());
    report.push_str(format!("{tab}|  dependencies:\n").as_str());
    for dep in class.class_deps {
        report.push_str(format!("{tab}|    {}\n", dep).as_str());
    }
    report.push_str(format!("{tab}|  nested classes:\n").as_str());
    for nes_class in class.nested_classes {
        let nes_class_string = get_string_with_nesting_level(nes_class, nes_level+1);
        report.push_str(format!("{nes_class_string}\n").as_str());
    }
    report.push_str(format!("{tab}|==========").as_str());
    report
}

impl Display for ClassDepsReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_string_with_nesting_level(self.clone(), 0))
    }
}

#[derive(Debug, Clone)]
pub struct PackageDepsReport {
    pub package_name: String,
    pub package_deps: Vec<String>
}