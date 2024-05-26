#[macro_use]
extern crate rocket;
use diesel::r2d2::{ self, ConnectionManager };
use rocket::tokio;
use rocket::tokio::time::sleep;
use rocket::tokio::time::Duration;
use rocket::tokio::task::JoinHandle;
// use rocket::fairing::AdHoc;

use diesel::prelude::*;
use diesel::PgConnection;

use schema::jobs::dsl::*;
use chrono::DateTime;

use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use cron::Schedule;
pub mod schema;

use schema::jobs::dsl::jobs as jobs_table;

pub fn establish_connection() -> DbPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = schema::jobs)]
pub struct Job {
    pub id: i32,
    pub schedule: String, // cron expression
    pub next_run: DateTime<chrono::Utc>,
}

impl Job {
    pub fn due_jobs(
        conn: &mut PgConnection,
        now: chrono::DateTime<chrono::Utc>
    ) -> Result<Vec<Job>, diesel::result::Error> {
        use schema::jobs::dsl::*;
        jobs.filter(next_run.le(now)).load::<Job>(conn)
    }
}

fn next_run_from_cron(
    other_schedule: &str,
    now: chrono::DateTime<chrono::Utc>
) -> chrono::DateTime<chrono::Utc> {
    let new_schedule = Schedule::from_str(other_schedule).unwrap();
    let mut iter = new_schedule.upcoming(chrono::Utc);
    iter.next().unwrap_or(now)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    let db_pool = establish_connection();

    let _scheduler_task: JoinHandle<_> = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            let now = chrono::Utc::now();

            if let Ok(jobs_to_run) = Job::due_jobs(&mut db_pool.get().unwrap(), now) {
                for mut job in jobs_to_run {
                    // Execute job logic (send notifications, etc.)
                    println!("Job {} executed at {:?}", job.id, job.next_run);

                    job.next_run = next_run_from_cron(&job.schedule, now);
                    diesel
                        ::update(jobs_table.find(job.id))
                        .set(next_run.eq(job.next_run))
                        .execute(&mut db_pool.get().unwrap())
                        .expect("Error updating job");
                }
            } else {
                eprintln!("Error fetching due jobs");
            }
        }
    });

    rocket::build().mount("/", routes![index])
    // .attach(
    //     AdHoc::on_shutdown("Shutdown Scheduler", |_|
    //         Box::pin(async move {
    //             scheduler_task.abort();
    //             if scheduler_task.await.is_err() {
    //                 // Handle scheduler shutdown error
    //             }
    //         })
    //     )
    // )
}
