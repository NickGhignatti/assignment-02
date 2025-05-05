use std::fs::read_dir;
use crate::common::types::{ClassDepsReport, PackageDepsReport, ProjectDepsReport};
use tokio::{fs::File, io::AsyncReadExt};
use tree_sitter::{Parser, Language, Node};
use walkdir::WalkDir;

pub async fn get_class_dependencies(class_src_file: String) -> Result<Vec<ClassDepsReport>, String> {
    let mut file = match File::open(class_src_file).await {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to read file: {}", e)),
    };

    // Create a Tree-sitter parser and set the Java language.
    let mut parser = Parser::new();
    let language: Language = Language::from(tree_sitter_java::LANGUAGE);
    parser.set_language(&language)
        .expect("Error loading Java grammar");

    let tree = parser.parse(&contents, None)
        .expect("Failed to parse the Java source");

    let root = tree.root_node();

    let classes = collect_all_classes(&root, &contents);

    Ok(classes)
}

fn collect_all_classes(node: &Node, code: &str) -> Vec<ClassDepsReport> {
    let mut classes = Vec::new();

    // Iterate only over *named* children of `node`
    for i in 0..node.named_child_count() {
        let child = node.named_child(i).unwrap();

        if child.kind() == "class_declaration" {
            let name_node = child
                .child_by_field_name("name")
                .expect("class_declaration without name");
            let class_name = name_node
                .utf8_text(code.as_bytes())
                .expect("Failed to read class name")
                .to_string();

            // Recurse into the body to find its direct nested classes
            let nested = if let Some(body) = child.child_by_field_name("body") {
                collect_all_classes(&body, code)
            } else {
                Vec::new()
            };

            // gather in-class deps
            let file_dependencies = collect_file_imports(&node, code);
            let class_dependencies = filter_dependencies(collect_class_dependencies(&child, code));
            classes.push(ClassDepsReport {
                class_name,
                class_deps: [file_dependencies, class_dependencies].concat(),
                nested_classes: nested,
            });
        }
    }

    classes
}

fn collect_file_imports(root: &Node, code: &str) -> Vec<String> {
    let mut dependencies = Vec::new();

    for i in 0..root.named_child_count() {
        let child = root.named_child(i).unwrap();

        // 2. Match any `import_declaration` node.
        if child.kind() == "import_declaration" {
            // 3. First named child is the path (a scoped_identifier).
            if let Some(path_node) = child.named_child(0) {
                let mut path = path_node
                    .utf8_text(code.as_bytes())
                    .unwrap()
                    .to_string();

                // 4. Handle wildcard imports: second named child might be `*`
                if child.named_child(1)
                    .map(|n| n.kind())
                    .unwrap_or("") == "asterisk"
                {
                    path.push_str(".*");
                }

                // 5. Handle static imports (literal "static" appears as an unnamed child)
                if child.child_by_field_name("static").is_some() ||
                    child.child_by_field_name("static").is_some()
                {
                    // prepend for clarity
                    path = format!("static {}", path);
                }

                dependencies.push(path);
            }
        }
    }
    dependencies
}

fn collect_class_dependencies(class_node: &Node, code: &str) -> Vec<String> {
    let mut deps = Vec::new();

    // 1. extends
    if let Some(superc) = class_node.child_by_field_name("superclass") {
        let n = superc.child_by_field_name("name").unwrap_or(superc);
        deps.push(n.utf8_text(code.as_bytes()).unwrap().to_string());
    }

    // 2. implements
    if let Some(interfaces) = class_node.child_by_field_name("super_interfaces") {
        for j in 0..interfaces.named_child_count() {
            let iface = interfaces.named_child(j).unwrap();
            deps.push(iface.utf8_text(code.as_bytes()).unwrap().to_string());
        }
    }

    // 3. fields, methods, params, new expressions
    let cursor = class_node.child_by_field_name("body").expect("no body");

    for i in 0..cursor.child_count() {
        let nd = match cursor.child(i) {
            Some(x) => x,
            None => continue,
        };
        match nd.kind() {
            "field_declaration"
            | "constructor_declaration"
            | "object_creation_expression" => {
                if let Some(t) = nd.child_by_field_name("type")
                {
                    match resolve_field(nd, vec!["declarator", "value", "type"]) {
                        Ok(x) => deps.push(x.utf8_text(code.as_bytes()).unwrap().to_string()),
                        Err(_) => (),
                    }
                    deps.push(t.utf8_text(code.as_bytes()).unwrap().to_string());
                }
            },
            "method_declaration" => {

                if let Some(t) = nd.child_by_field_name("type")
                {
                    match resolve_field(nd, vec!["declarator", "value", "type"]) {
                        Ok(x) => deps.push(x.utf8_text(code.as_bytes()).unwrap().to_string()),
                        Err(_) => (),
                    }
                    deps.push(t.utf8_text(code.as_bytes()).unwrap().to_string());
                }

                if let Some(p) = nd.child_by_field_name("parameters")
                {
                    for x in 0..p.child_count() {
                        if p.child(x).unwrap().kind() == "formal_parameter" {
                            if let Some(t) = p.child(x).unwrap().child_by_field_name("type")
                            {
                                match resolve_field(t, vec!["declarator", "value", "type"]) {
                                    Ok(x) => deps.push(x.utf8_text(code.as_bytes()).unwrap().to_string()),
                                    Err(_) => (),
                                }
                                deps.push(t.utf8_text(code.as_bytes()).unwrap().to_string());
                            }
                        }
                    }
                }

                if let Some(meth_body) = nd.child_by_field_name("body") {
                    for j in 0..meth_body.child_count() {
                        let body_field = match meth_body.child(j) {
                            Some(x) => x,
                            None => continue,
                        };
                        match body_field.kind() {
                           "local_variable_declaration"
                            | "return_statement" => {
                                if let Some(t) = body_field.child_by_field_name("type")
                                {
                                    match resolve_field(body_field, vec!["declarator", "value", "type"]) {
                                        Ok(x) => deps.push(x.utf8_text(code.as_bytes()).unwrap().to_string()),
                                        Err(_) => (),
                                    }
                                    deps.push(t.utf8_text(code.as_bytes()).unwrap().to_string());
                                }
                            },
                            "expression_statement" => {
                                for i in 0..body_field.child_count() {
                                    let expression_node = body_field.child(i).unwrap();
                                    if expression_node.kind() == "method_invocation" {
                                        for j in 0..expression_node.child_count() {
                                            let obj_creation_node = expression_node.child(j).unwrap();
                                            if obj_creation_node.kind() == "object_creation_expression" {
                                                if let Some(t) = obj_creation_node.child_by_field_name("type")
                                                {
                                                    match resolve_field(obj_creation_node, vec!["declarator", "value", "type"]) {
                                                        Ok(x) =>
                                                            deps.push(x.utf8_text(code.as_bytes()).unwrap().to_string()),
                                                        Err(_) => (),
                                                    }
                                                    deps.push(t.utf8_text(code.as_bytes()).unwrap().to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => ()
                        }
                    }
                }
            },
            _ => {}
        }
    }

    deps.sort();
    deps.dedup();
    deps
}

fn filter_dependencies(dependencies: Vec<String>) -> Vec<String> {
    let prims = [
        "byte", "short", "int", "long",
        "float", "double", "boolean", "char",
        "void",
    ];

    dependencies.into_iter()
        .filter(|ty| !prims.contains(&ty.as_str()))
        .collect()
}

fn resolve_field<'a>(node: Node<'a>, fields: Vec<&'a str>) -> Result<Node<'a>, String> {
    let mut return_node: Node = node.clone();
    for f in fields {
        if let Some(n) = return_node.child_by_field_name(f) {
            return_node = n;
        } else {
            return Err("Some of the fields name are wrong!".parse().unwrap());
        }
    }
    Ok(return_node)
}

pub async fn get_package_dependencies(package_folder: String) -> Result<PackageDepsReport, String> {
    let paths = match read_dir(package_folder.clone()) {
        Ok(p) => p,
        _ => return Err(String::from("Invalid folder"))
    };

    let p_folder = package_folder.clone();
    let mut dependencies: Vec<String> = Vec::new();
    for path in paths {
        let file_name = path.unwrap().file_name().into_string().unwrap();
        if file_name.contains(".java") {
            let file = format!("{p_folder}/{file_name}");
            match get_class_dependencies(file).await {
                Ok(classes) => {
                    for mut class in classes {
                        dependencies.append(&mut class.class_deps);
                    }
                }
                Err(e) => println!("Err in getting package deps: {} for file {}", e, file_name),
            }
        }
    }

    dependencies.sort();
    dependencies.dedup();

    Ok(PackageDepsReport {
        package_name: package_folder,
        package_deps: dependencies
    })
}

pub async fn get_project_dependencies(project_folder: String) -> Result<ProjectDepsReport, String> {
    let mut dependencies: Vec<String> = Vec::new();
    for entry in WalkDir::new(project_folder.clone()).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.path().file_name().unwrap().to_str().unwrap();
        if entry.path().is_file() && file_name.contains(".java") {
            match get_class_dependencies(entry.path().to_str().unwrap().to_string()).await {
                Ok(vector) => for c in vector {
                    dependencies.append(&mut c.get_dependencies());
                },
                Err(e) => return Err(e)
            }
        }
    }

    dependencies.sort();
    dependencies.dedup();

    Ok(ProjectDepsReport {
        project_folder,
        project_deps: dependencies
    })
}