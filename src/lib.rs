use std::{error::Error, time::Duration};

use chrono::{Duration, Utc};
use sqlx::{Database, Executor, PgPool, Pool, Postgres};
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Config{
    pub database_url : String,
    pub announce_offsets : Vec<Duration>,
}


pub async fn run(config : Config) -> Result<(), Box<dyn Error>>{
    let pool = create_pool(&config).await?;
    let mut sched = JobScheduler::new().await.unwrap();

    schedule_tasks(&pool, &mut sched, &config).await?;

    sched.start().await?;

    sleep(Duration::new(1000, 0)).await;
    Ok(())
}

async fn schedule_tasks<'e, 'c, E>(pool : &E, sched : &mut JobScheduler, config : &Config) -> Result<(), Box<dyn Error>>
where 
    E: 'e + Executor<'c, Database = Postgres>
{
    let events = get_all_events(pool).await?;
    
    let mut pingable_events = vec![];
    for event in &events{
        pingable_events.push(process_event(pool, event, config).await?);
    }

    for pingable_event in pingable_events.into_iter().flatten(){
        let ttt = Utc::now() - pingable_event.ping_time;
        
        println!("Scheduling task in {ttt:?}");

        let job = Job::new_one_shot(ttt.to_std().unwrap(), |_, _|{
            println!("{pingable_event:?}");
        })?;

        sched.add(job).await?;
    }

    Ok(())
}

async fn create_pool(config : &Config) -> Result<Pool<Postgres>, sqlx::Error>{
    PgPool::connect(&config.database_url).await    
} 

struct Event{
    race_name : String,
    race_time : chrono::DateTime<Utc>,
    race_number : i32,
    league_id : i32,
}

async fn get_all_events<'e, 'c, E>(pool : E) -> Result<Vec<Event>, sqlx::Error>
where
    E: 'e + Executor<'c, Database = Postgres>,
{
    let events = sqlx::query!("SELECT * FROM races").fetch_all(pool).await?;

    Ok(events.iter().map(|r| {Event{
        race_name: r.name.clone(),
        race_time: r.time,
        race_number: r.number,
        league_id: r.league_id
    }}).collect())
}

#[derive(Debug)]
struct PingableEvent{
    race_name : String,
    ping_time : chrono::DateTime<Utc>,
    race_number : i32,
    league_name : String,
    ping_channel : String,
    ping_role : String,
}

async fn process_event<'e, 'c, E>(pool : E, event: &Event, config : &Config) -> Result<Vec<PingableEvent>, Box<dyn Error>>
where 
   E: 'e + Executor<'c, Database = Postgres>
{
    let result = sqlx::query!("SELECT name, roleid, channelid FROM leagues WHERE id=$1", event.league_id).fetch_one(pool).await?;
    
    Ok(config.announce_offsets.iter().map(|offset| {PingableEvent{
        race_name: event.race_name,
        race_number: event.race_number,
        ping_time: event.race_time - *offset,
        league_name: result.name,
        ping_channel: result.channelid,
        ping_role: result.roleid,
    }}).collect())

}
