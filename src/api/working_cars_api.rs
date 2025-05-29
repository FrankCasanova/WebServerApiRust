use std::collections::HashMap;
use uuid::Uuid;
use crate::models::model_garage::GarageModel;
use serde::{Serialize, Deserialize};

use actix_web::{ post, delete, get, web, HttpResponse, Responder};
use diesel::{r2d2::{self, ConnectionManager}, PgConnection};
use serde_json::json;

use crate::models::model_working_cars::{NewWorkingCarHandler, WorkingCarsModel};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AssignCarRequest {
    pub car_to_repair: Uuid,
    pub garage_id: Option<i64>, // este campo es importante para poder elegir el garage/taller al que se asigna el coche
}

// Esta función ha sido refactorizada.
#[post("/api/workingcar/assingcar")]
pub async fn assign_car_to_repair(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>, 
    item: web::Json<AssignCarRequest>
) -> impl Responder {
    let mut conn = pool.get().expect("Problemas al obtener la conexion");
    
    // Comprobación para saber si un coche ha sido previamente asignado a un taller
    let working_cars = WorkingCarsModel::get_all_working_cars(&mut conn).unwrap_or_default();
    let is_assigned = working_cars.iter().any(|c| c.car_to_repair == item.car_to_repair);
    if is_assigned {
        return HttpResponse::BadRequest().body("This car is already assigned to a garage.");
    }

    // Traemos los garages disponibles
    // Si no hay garages, devolvemos un error
    let garages = GarageModel::get_garages(&mut conn).unwrap_or_default();

    // Asignación del coche al taller asignado por el usuario en el frontend.
    // El frontend tiene un dropdown con los garages disponibles.
    // Este código está un poco feo, pero el If let Some() es lo que tenía más fresco en mi cabeza.
    // quizá puede hacerse más bonito con un match o algo así.
    if let Some(selected_garage_id) = item.garage_id {
        if let Some(garage) = garages.iter().find(|g| g.id == selected_garage_id) {
            let count = working_cars.iter().filter(|c| c.assigned_garage == garage.id).count() as i32;
            if count < garage.capacity {
                let new_working_car = NewWorkingCarHandler {
                    assigned_garage: garage.id,
                    car_to_repair: item.car_to_repair,
                };
                match web::block(move || WorkingCarsModel::add_new_working_car(&mut conn, &new_working_car)).await {
                    Ok(data) => {
                        let data = data.unwrap();
                        return HttpResponse::Ok().json(json!(data));
                    }
                    Err(err) => return HttpResponse::Ok().body(err.to_string()),
                }
            } else {
                return HttpResponse::BadRequest().body("El Garage seleccionado está lleno.");
            }
        } else {
            return HttpResponse::BadRequest().body("El Garage seleccionado no existe.");
        }
    } else {
        return HttpResponse::BadRequest().body("No has seleccionado un Garage.");
    }
}

#[get("/api/garages/available")]
pub async fn get_available_garages(pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>) -> impl Responder {
    let mut conn = pool.get().expect("Problemas al obtener la conexion");
    let garages = GarageModel::get_garages(&mut conn).unwrap_or_default();
    let working_cars = WorkingCarsModel::get_all_working_cars(&mut conn).unwrap_or_default();
    let available: Vec<_> = garages.into_iter().filter(|g| {
        let count = working_cars.iter().filter(|c| c.assigned_garage == g.id).count() as i32;
        count < g.capacity
    }).collect();
    HttpResponse::Ok().json(json!(available))
}

#[post("/api/workingcar/getworkingcars")]
pub async fn get_all_working_cars_assigned(pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>) -> impl Responder {
    let mut conn = pool.get().expect("Problemas al obtener la conexion");
    match web::block(move || WorkingCarsModel::get_all_working_cars(&mut conn)).await {
        Ok(data) => {
            let data = data.unwrap();
            HttpResponse::Ok().json(json!(data))
        }
        Err(err) => HttpResponse::Ok().body(err.to_string()),
    }
}

#[delete("/api/workingcar/carrepaired")]
pub async fn repaired_car(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>, 
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let mut conn = pool.get().expect("Problemas al obtener la conexion");

    // Aceptamos el coche en base a su UUID, esto es importante para que no se pueda borrar un coche que no existe.
    // también para evitar problemas de concurrencia, ya que el UUID es único.
    // también se evita que se pueda asignar el mismo coche a dos talleres diferentes o al mismo.
    let car_id_str = query.get("car_to_repair");
    if car_id_str.is_none() {
        return HttpResponse::BadRequest().body("Missing car_to_repair parameter");
    }
    let car_id = match Uuid::parse_str(car_id_str.unwrap()) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid car_to_repair UUID"),
    };
    // Borramos el coche de la tabla WorkingCars con una nueva función delete_by_car_to_repair
    // la otra función queda ahora huerfana pero sin borrar por si se necesita para algo en un futuro.
    // conforme se avancen las versiones o algo pues se podría borrar definitivamente.
    match web::block(move || WorkingCarsModel::delete_by_car_to_repair(&mut conn, &car_id)).await {
        Ok(data) => {
            let data = data.unwrap();
            HttpResponse::Ok().json(json!(data))
        }
        Err(err) => HttpResponse::Ok().body(err.to_string()),
    }
}