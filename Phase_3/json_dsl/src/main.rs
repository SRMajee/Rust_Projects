use json_dsl::json;

fn main() {
    println!("=== Phase 3: Declarative JSON DSL Demo ===");

    let user_id = 1042;
    let username = "alex_rustacean";
    let is_admin = true;
    let base_score = 85.0;
    let bonus = 12.5;

    // Construct a deeply nested JSON document using declarative syntax
    let payload = json!({
        "id": user_id,
        "username": username,
        "is_admin": is_admin,
        "final_score": (base_score + bonus),
        "profile": {
            "title": "Systems Engineer",
            "bio": null,
            "skills": ["Rust", "Macros", "Systems Programming"],
            "settings": {
                "theme": "dark",
                "notifications": true,
                "retries": 3,
            }
        },
        "tags": ["core", "verified",],
        "metadata": {}
    });

    println!("\n--- Serialized JSON (Display) ---");
    println!("{}", payload);

    println!("\n--- Rust AST (Debug) ---");
    println!("{:#?}", payload);

    // Demonstrate typed traversal
    println!("\n--- AST Traversal ---");
    if let Some(profile) = payload.get("profile") {
        if let Some(skills) = profile.get("skills") {
            if let Some(first_skill) = skills.get_index(0) {
                println!("First skill: {}", first_skill.as_str().unwrap_or("N/A"));
            }
        }
        if let Some(bio) = profile.get("bio") {
            println!("Bio is null: {}", bio.is_null());
        }
    }
}
