use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

/// Fully-qualified Java name
type FqName = String;
/// Map from a class to the set of other classes it depends on
type DepGraph = HashMap<FqName, HashSet<FqName>>;

// Java keywords, primitives, etc., to ignore
lazy_static! {
    static ref EXCLUDED: HashSet<&'static str> = {
        let words = [
            // primitives
            "int","long","short","byte","char","float","double","boolean","void",
            // keywords
            "new","return","public","protected","private","static","final","abstract",
            // control flow
            "if","else","for","while","switch","case","default","break","continue",
            "try","catch","finally","throw","throws","synchronized",
        ];
        words.iter().copied().collect()
    };
}

/// Walk directory, find .java files, and build the graph
pub fn build_dependency_graph(root: &Path) -> io::Result<DepGraph> {
    // Regex for package/import
    let pkg_re = Regex::new(r"^\s*package\s+([\w\.]+)\s*;").unwrap();
    let imp_re = Regex::new(r"^\s*import\s+([\w\.]+)(?:\.\*)?\s*;").unwrap();
    // new Foo()
    let new_re = Regex::new(r"\bnew\s+([\w<>.\[\]]+)").unwrap();
    // declarations: Type name;
    let decl_re = Regex::new(r"\b([\w<>.\[\]]+)\s+\w+\s*(?:[=;,(])").unwrap();
    // method signatures, capturing entire param list in group 2
    let sig_re = Regex::new(
        r"[\w<>.\[\]]+\s+\w+\s*\(([^)]*)\)"
    ).unwrap();

    let mut graph = DepGraph::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path: PathBuf = entry.path().to_path_buf();
        if path.extension().and_then(|s| s.to_str()) == Some("java") {
            process_java_file(
                &path,
                &pkg_re,
                &imp_re,
                &new_re,
                &decl_re,
                &sig_re,
                &mut graph,
            )?;
        }
    }
    Ok(graph)
}

/// Normalize a raw type string: remove generics, array markers, var names
fn normalize_type(raw: &str) -> Option<String> {
    // strip generics: Foo<Bar> => Foo
    let without_generics = raw.split('<').next().unwrap_or(raw);
    // strip array: Foo[] => Foo
    let without_array = without_generics.trim_end_matches("[]");
    // trim whitespace
    let ty = without_array.trim();
    if ty.is_empty() ||
        EXCLUDED.contains(ty) {
        None
    } else {
        Some(ty.to_string())
    }
}

fn process_java_file(
    path: &Path,
    pkg_re: &Regex,
    imp_re: &Regex,
    new_re: &Regex,
    decl_re: &Regex,
    sig_re: &Regex,
    graph: &mut DepGraph,
) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut package = String::new();
    let class_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("<unknown>")
        .to_string();
    let mut deps: HashSet<String> = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        // package
        if package.is_empty() {
            if let Some(caps) = pkg_re.captures(&line) {
                package = caps[1].to_string();
                continue;
            }
        }
        // imports
        if let Some(caps) = imp_re.captures(&line) {
            if let Some(ty) = normalize_type(&caps[1]) {
                deps.insert(ty);
            }
            continue;
        }
        // new Foo<Bar>()
        for caps in new_re.captures_iter(&line) {
            if let Some(ty) = normalize_type(&caps[1]) {
                deps.insert(ty);
            }
        }
        // declarations: Foo name;
        for caps in decl_re.captures_iter(&line) {
            if let Some(ty) = normalize_type(&caps[1]) {
                deps.insert(ty);
            }
        }
        // method signatures: capture inside parentheses
        if let Some(caps) = sig_re.captures(&line) {
            let params = &caps[1]; // e.g. "E e, List<String> xs"
            for raw_param in params.split(',') {
                // split on whitespace, first token is type, rest is var name
                let parts: Vec<_> = raw_param.trim().split_whitespace().collect();
                if !parts.is_empty() {
                    if let Some(ty) = normalize_type(parts[0]) {
                        deps.insert(ty);
                    }
                }
            }
        }
    }

    // build fully-qualified class name
    let fqcn = if package.is_empty() {
        class_name.clone()
    } else {
        format!("{}.{}", package, class_name)
    };
    graph.entry(fqcn).or_default().extend(deps);

    Ok(())
}
