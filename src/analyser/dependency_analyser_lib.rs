use std::collections::HashSet;
use crate::common::types::ClassDepsReport;
use tokio::{fs::File, io::AsyncReadExt};
use tree_sitter::{Parser, Language};

pub async fn get_class_dependencies(class_src_file: String) -> Result<ClassDepsReport, String> {
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

    println!("{}", classes[0]);

    Err("Not implemented yet".to_string())
}

/// Recursively collects all classes that are children of the node (with depth 0 so direct).
fn collect_all_classes(node: &tree_sitter::Node, code: &str) -> Vec<ClassDepsReport> {
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
                class_deps: class_dependencies,
                nested_classes: nested,
            });
        }
    }

    classes
}

fn collect_file_imports(root: &tree_sitter::Node, code: &str) -> Vec<String> {
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

fn collect_class_dependencies(class_node: &tree_sitter::Node, code: &str) -> Vec<String> {
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
    let mut cursor = class_node.walk();
    loop {
        let nd = cursor.node();
        match nd.kind() {
            "field_declaration"
            | "method_declaration"
            | "constructor_declaration"
            | "object_creation_expression" => {
                if let Some(t) = nd.child_by_field_name("type")
                    .or_else(|| nd.child_by_field_name("type_identifier"))
                {
                    deps.push(t.utf8_text(code.as_bytes()).unwrap().to_string());
                }
            }
            _ => {}
        }
        if cursor.goto_first_child() {
            continue;
        }
        if !cursor.goto_next_sibling() {
            // ascend until able to goto_next_sibling
            while cursor.goto_parent() && !cursor.goto_next_sibling() {}
            if cursor.node().is_missing() { break; }
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