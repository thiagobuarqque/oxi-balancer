use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::{ModifyKind, DataChange}};
use std::path::Path;

pub enum Algorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnection,
    WeightedLeastConnection,
    ResourceBased
}

pub struct Config {
    algorithm: Algorithm,
    addresses: Vec<String>
}

pub async fn setup_config_provider() -> notify::Result<()> {
    let mut watcher = RecommendedWatcher::new(

        |res: Result<Event, notify::Error>| match res {
            Ok(event) => {
                if is_save_event(&event) {
                    println!("File saved! Paths: {:?}", event.paths);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        },
        notify::Config::default(),
    )?;

    watcher.watch(Path::new("/home/evry/Desktop/repositories/oxi-balance/config.yaml"), RecursiveMode::NonRecursive)?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn is_save_event(event: &Event) -> bool {
    match event.kind {
        // Captures standard content modification or generic modifications
        EventKind::Modify(ModifyKind::Data(DataChange::Any)) |
        EventKind::Modify(ModifyKind::Any) => true,

        // Some editors save by creating a temporary file and renaming it over the old one
        EventKind::Modify(ModifyKind::Name(_)) => true,

        _ => false,
    }
}
