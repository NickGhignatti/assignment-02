mod common;
mod analyser;

use analyser::dependency_analyser_lib::get_class_dependencies;
use crate::analyser::dependency_analyser_lib::{get_package_dependencies, get_project_dependencies};

#[tokio::main]
async fn main() {
    println!("Starting program:");

    match get_class_dependencies("src/test_files/src/main/java/pcd/ass02/MyClass.java".to_string()).await {
        Ok(report) => {
            for r in report {
                println!("{r}");
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    // match get_package_dependencies("src/test_files".to_string()).await {
    //     Ok(report) => println!("{:?}", report),
    //     Err(e) => println!("{}", e)
    // }
    // match get_project_dependencies("src/test_files".to_string()).await {
    //     Ok(report) => println!("{:?}", report),
    //     Err(e) => println!("{}", e)
    // }
}
