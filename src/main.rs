mod common;
mod analyser;

use analyser::dependency_analyser_lib::get_class_dependencies;

#[tokio::main]
async fn main() {
    println!("Starting program:");

    match get_class_dependencies("src/test_files/HelloWorld.java".to_string()).await {
        Ok(report) => {
            for r in report {
                println!("{r}");
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }
}
