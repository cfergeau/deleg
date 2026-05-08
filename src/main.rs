mod models;

use models::Person;

fn main() {
    let person = Person::new(
        "John".to_string(),
        "Doe".to_string(),
        "Developer".to_string(),
    );
    println!("Created person: {:?}", person);
}
