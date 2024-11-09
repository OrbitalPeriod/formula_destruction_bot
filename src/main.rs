use chrono::Duration;
use dotenv::dotenv;
use FormulaDestructionBot::{run, Config};


#[tokio::main]
async fn main() {
    let config = get_config();
    if let Err(e) = run(config).await{
        eprintln!("{e:?}");
    }
}

fn get_config() -> Config{
    let _ =dotenv();
    
    let database_url = std::env::var("DATABASE_URL").expect("Database URL not found...");

    Config{
        database_url,
        announce_offsets: vec![Duration::new(10, 0).expect("???")]
    }
}
