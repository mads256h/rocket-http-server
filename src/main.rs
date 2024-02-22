#[macro_use] extern crate rocket;


use rocket::{Rocket, Build};
use rocket::fairing::{self, AdHoc};
use rocket::serde::json::Json;
use rocket::serde::{Serialize, Deserialize};

use rocket_db_pools::{sqlx, Database, Connection};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Task {
    id: i64,
    timespan: Timespan,
    duration: i64,
    effect: f64,
    device_id: i64,
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Timespan {
    start: i64,
    end: i64,
}


#[get("/")]
async fn hello(mut db: Connection<Db>) -> Result<Json<Vec<Task>>> {
    let tasks = sqlx::query!(r#"
        SELECT id, timespan_start, timespan_end, duration, effect, device_id
        FROM Tasks
    "#).fetch_all(&mut **db).await?;

    let my_tasks = tasks.iter().map(|t| Task{
        id: t.id,
        timespan: Timespan { start: t.timespan_start, end: t.timespan_end },
        duration: t.duration,
        effect: t.effect,
        device_id: t.device_id
    }).collect();
    Ok(Json(my_tasks))
}

#[get("/create")]
async fn create(mut db: Connection<Db>) -> Result<Json<Task>> {
    let mut new_task = Task{
        id: -1,
        timespan: Timespan { start: 1992, end: 2000 }, 
        duration: 5 * 60 * 1000, 
        effect: 1000.0, 
        device_id: 1234 };

    let id = sqlx::query_scalar!(r#"
    INSERT INTO Tasks (timespan_start, timespan_end, duration, effect, device_id)
    VALUES (?, ?, ?, ?, ?)
    RETURNING id
    "#,
        new_task.timespan.start,
        new_task.timespan.end,
        new_task.duration,
        new_task.effect,
        new_task.device_id
    ).fetch_one(&mut **db).await?;

    new_task.id = id;

    Ok(Json(new_task))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::on_ignite("Main stage", |rocket| async {
            rocket.attach(Db::init())
            .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
            .mount("/", routes![hello, create])
        }))
}
