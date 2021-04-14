use crate::context::{Metrics, SyncContext};
use crate::http::http_post;
use crate::model::Tile;
use async_recursion::async_recursion;
use std::time::Duration;
use url::Url;

pub struct Explorer {
    url: Url,
    timeout: Duration,
    client: surf::Client,
    min_amount: u64,
    binary_enabled: bool,
    flat_threshold: u64,
    flat_size: u64,
    sync: SyncContext,
}

impl Explorer {
    pub fn new(
        url: Url,
        timeout: Duration,
        min_amount: u64,
        binary_enabled: bool,
        flat_threshold: u64,
        flat_size: u64,
        sync: SyncContext,
    ) -> Explorer {
        Explorer {
            url,
            timeout,
            client: surf::Client::new(),
            min_amount,
            binary_enabled,
            flat_threshold,
            flat_size,
            sync,
        }
    }

    pub async fn check_tile(&self, tile: Tile) -> Tile {
        let result: Tile;
        loop {
            self.sync.explore_rate_limiter().await;
            //println!("{}",tile.area.size_x);
            result = match http_post(
                &self.url,
                self.timeout,
                tile.area.clone(),
                &self.client,
                self.sync.clone(),
            )
            .await
            {
                Ok(t) => t,
                Err(_) => {
                    continue;
                }
            };

            self.sync
                .metrics_sender
                .send(Metrics::new_explore(
                    result.is_single_point() && result.amount > 0,
                    result.area.pos_x,
                    result.area.pos_y,
                    result.area.size_x,
                    result.area.size_y,
                ))
                .await
                .unwrap();

            break;
        }
        result
    }

    #[async_recursion]
    pub async fn search(&self, tile: Tile, mut top: bool) -> Vec<Tile> {
        let mut result = Vec::new();
        let mut tile = tile;
        if tile.amount == 0 {
            tile = self.check_tile(tile).await;
        }
        let mut min_amount = 1;
        if top {
            min_amount = self.min_amount;
            top = false
        }
        if tile.has_treasures(min_amount) {
            if tile.is_single_point() {
                result.push(tile);
                return result;
            }
            if tile.area.size_x > self.flat_threshold && self.binary_enabled {
                    let (left, mut right) = tile.split();
                    let mut left = self.search(left, top).await;
                    left.iter().for_each(|t| tile.amount -= t.amount);
                    result.append(&mut left);
                    if tile.amount > 0 {
                        right.amount = tile.amount;
                        let mut right = self.search(right, top).await;
                        result.append(&mut right);
                    }
            } else {
                let tile_size = if tile.area.size_x <= self.flat_size {
                    1
                } else if !self.binary_enabled && tile.area.size_x>self.flat_threshold{
                    self.flat_threshold
                } else {
                    self.flat_size
                };

                let iter = tile.split_to_tiles(tile_size).into_iter();
                let len = iter.len();
                let mut cur = 1;

                for mut t in iter {
                    if cur == len {
                        t.amount = tile.amount;

                        if tile_size == 1 {
                            self.sync
                                .metrics_sender
                                .send(Metrics::new_calculated_explore(
                                    true,
                                    t.area.pos_x,
                                    t.area.pos_y,
                                ))
                                .await
                                .unwrap();
                        }


                        result.push(t);
                        break;
                    }
                    let mut t = self.search(t, top).await;
                    t.iter().for_each(|t| tile.amount -= t.amount);
                    result.append(&mut t);
                    if tile.amount == 0 {
                        break;
                    }
                    cur += 1;
                }
            }
        }
        result
    }

    pub async fn start(self) {
        loop {
            let initial_area = self.sync.area_receiver.recv().await.unwrap();
            let tiles = self.search(initial_area, true).await;
            for tile in tiles {
                self.sync.tile_sender.send(tile).await.unwrap();
            }
        }
    }
}
