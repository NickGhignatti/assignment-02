mod common;
mod analyser;

use analyser::dependency_analyser_lib::get_class_dependencies;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    match get_class_dependencies("src/test_files/HelloWorld.java".to_string()).await {
        Ok(report) => println!("Dependencies: {}", report),
        Err(e) => eprintln!("Error: {}", e),
    }
}
