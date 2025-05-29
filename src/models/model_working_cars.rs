use super::super::schema::WorkingCars::dsl::*;
use super::super::schema::WorkingCars;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//Ahora los modelos
#[derive(Insertable,Debug)] 
#[diesel(table_name = WorkingCars)]
pub struct NewWorkingCar<'a>{
    pub assigned_garage: &'a i64,
    pub car_to_repair: &'a Uuid,
}

//Ahora el handler, lo necesitaremos para las apis
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NewWorkingCarHandler{
    pub assigned_garage: i64,
    pub car_to_repair: Uuid
}


//Ahora los modelos
#[derive(Queryable,Debug, Deserialize, Serialize)]
#[diesel(table_name = WorkingCars)]
pub struct WorkingCarsModel{
    pub id: i64,
    pub assigned_garage: i64,
    pub car_to_repair: Uuid
}

impl WorkingCarsModel{
    pub fn get_all_working_cars(conn:  &mut PgConnection)->Result<Vec<WorkingCarsModel>, diesel::result::Error>{
        let working_cars = WorkingCars.load::<WorkingCarsModel>(conn);
        working_cars
    }

    pub fn add_new_working_car(conn:  &mut PgConnection, new_working_car: &NewWorkingCarHandler)-> Result<WorkingCarsModel,diesel::result::Error>{
        let new_car = NewWorkingCar{
            assigned_garage: &new_working_car.assigned_garage,
            car_to_repair: &new_working_car.car_to_repair,
        };

        diesel::insert_into(WorkingCars::table)
            .values(new_car)
            .get_result::<WorkingCarsModel>(conn)
    }

    // Función huerfana, esta no es la que se usa en esta nueva implementación.
    pub fn repaired_car(conn:  &mut PgConnection, working_car_id: &i64) -> Result<usize, diesel::result::Error>{
        let repaired_car = diesel::delete(WorkingCars.filter(id.eq(working_car_id)))
        .execute(conn);
        repaired_car
    }

    // Esta es la nueva función, se parece a la anterior solo que en esta aceptamos el UUID del coche a reparar.
    pub fn delete_by_car_to_repair(conn: &mut PgConnection, car_uuid: &Uuid) -> Result<usize, diesel::result::Error> {
    let deleted = diesel::delete(WorkingCars.filter(car_to_repair.eq(car_uuid)))
        .execute(conn);
    deleted
}

    pub fn get_working_cars_assigned_to_garage(conn:  &mut PgConnection, garage_id: &i64)->Result<Vec<WorkingCarsModel>, diesel::result::Error>{
        let working_cars = WorkingCars.filter(assigned_garage.eq(garage_id)).load::<WorkingCarsModel>(conn);
        working_cars
    }
}

