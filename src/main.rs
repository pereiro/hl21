mod config;
mod context;
mod http;
mod model;
mod workers;

use crate::config::Config;
use crate::context::SyncContext;
use crate::model::{Area, Tile};
use crate::workers::accountant::Accountant;
use crate::workers::attorney::Attorney;
use crate::workers::explorer::Explorer;
use crate::workers::digger::Digger;
use crate::workers::statist::Statist;
use async_std::task;
use envconfig::Envconfig;
use std::io;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let config: Config = Config::init_from_env().unwrap();

    println!("{}", config);

    let mut context = SyncContext::new(config);
    context.init().await;

    let statist = Statist::new(config.statist_display_tick, context.clone());
    task::spawn(async move { statist.start().await });

    let ctx = context.clone();

    task::spawn(async move {
        for y in 0..config.world_size {
            for x in (0..config.world_size - config.search_initial_array_size)
                .into_iter()
                .step_by(config.search_initial_array_size as usize)
            {
                let tile = Tile {
                    amount: 0,
                    area: Area::new(x, y, config.search_initial_array_size, 1),
                };
                ctx.area_sender.send(tile).await.unwrap();
            }
        }
    });

    for _ in 0..config.search_explorers_num {
        let context = context.clone();
        let binary_explorer = Explorer::new(
            config.get_explore_url(),
            Duration::from_millis(config.http_timeout_ms),
            config.search_min_amount,
            config.search_binary_enabled,
            config.search_to_flat_threshold,
            config.search_flat_size,
            context.clone(),
        );
        task::spawn(async move { binary_explorer.start().await });
    }

    for _ in 0..config.attorneys_num {
        let attorney = Attorney::new(
            config.get_licenses_url(),
            Duration::from_millis(config.attorney_http_timeout_ms),
            config.attorney_license_min_cost,
            config.attorney_license_max_cost,
            config.attorney_free_license_probability,
            context.clone(),
        );
        task::spawn(async move { attorney.start().await });
    }

    for _ in 0..config.diggers_num {
        let digger = Digger::new(
            config.get_dig_url(),
            Duration::from_millis(config.http_timeout_ms),
            config.digger_min_depth,
            config.digger_max_depth,
            config.digger_min_depth_probability,
            context.clone(),
        );
        task::spawn(async move { digger.start().await });
    }

    for _ in 0..config.accountant_num {
        let accountant = Accountant::new(
            config.get_cash_url(),
            Duration::from_millis(config.accountant_http_timeout_ms),
            context.clone(),
        );
        task::spawn(async move { accountant.start().await });
    }

    loop {
        task::sleep(Duration::from_secs(config.phase2_start)).await;
        if config.enable_phased {
            context.switch_phase();
            println!("phase switched")
        }
    }
}
