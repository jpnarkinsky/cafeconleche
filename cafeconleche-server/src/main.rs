#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Mutex;

use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

// The type to represent the ID of a Recipe.
type ID = usize;

// We're going to store all of the Recipes here. No need for a DB.
type RecipeMap = Mutex<HashMap<ID, String>>;

#[derive(Serialize, Deserialize)]
struct Recipe {
    id: Option<ID>,
    name: String,
}

// TODO: This example can be improved by using `route` with multiple HTTP verbs.
#[post("/<id>", format = "json", data = "<Recipe>")]
fn new(id: ID, Recipe: Json<Recipe>, map: State<RecipeMap>) -> JsonValue {
    let mut hashmap = map.lock().expect("map lock.");
    if hashmap.contains_key(&id) {
        json!({
            "status": "error",
            "reason": "ID exists. Try put."
        })
    } else {
        hashmap.insert(id, Recipe.0.name);
        json!({ "status": "ok" })
    }
}

#[put("/<id>", format = "json", data = "<Recipe>")]
fn update(id: ID, Recipe: Json<Recipe>, map: State<RecipeMap>) -> Option<JsonValue> {
    let mut hashmap = map.lock().unwrap();
    if hashmap.contains_key(&id) {
        hashmap.insert(id, Recipe.0.name);
        Some(json!({ "status": "ok" }))
    } else {
        None
    }
}

#[get("/<id>", format = "json")]
fn get(id: ID, map: State<RecipeMap>) -> Option<Json<Recipe>> {
    let hashmap = map.lock().unwrap();
    hashmap.get(&id).map(|name| {
        Json(Recipe {
            id: Some(id),
            name: name.clone()
        })
    })
}

#[get("/", format = "json")]
fn list(map: State<RecipeMap>) -> Option<Json<Vec<Recipe>>> {
    let hashmap = map.lock().unwrap();
    Some(Json(hashmap.iter().map(|data| {
        Recipe {
            id: Some(*data.0),
            name: data.1.clone(),
        }
    }).collect()))
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

fn rocket() -> rocket::Rocket {
    let mut map = HashMap::<ID, String>::new();
    map.insert(usize::try_from(1).unwrap(), "Blooming Espresso".to_string());
    map.insert(usize::try_from(2).unwrap(), "Londinium".to_string());

    let mtx = Mutex::new(map);
    rocket::ignite()
        .mount("/Recipe", routes![new, update, get, list])
        .register(catchers![not_found])
        .manage(mtx)
}

fn main() {
    rocket().launch();
}
