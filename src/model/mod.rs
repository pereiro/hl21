use std::fmt::{Display, Formatter};
use std::iter::FromIterator;
use surf::http::convert::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Balance {
    balance: u32,
    wallet: Vec<u32>,
}

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = "[".to_string();
        for coin in &self.wallet {
            s = s + "," + &*coin.to_string()
        }
        s += "]";
        write!(f, "balance: {}, wallet: {}", self.balance, s)
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Area {
    #[serde(rename = "posX")]
    pub pos_x: u64,
    #[serde(rename = "posY")]
    pub pos_y: u64,
    #[serde(rename = "sizeX")]
    pub size_x: u64,
    #[serde(rename = "sizeY")]
    pub size_y: u64,
}

impl Area {
    pub fn new(pos_x: u64, pos_y: u64, size_x: u64, size_y: u64) -> Area {
        Area {
            pos_x,
            pos_y,
            size_x,
            size_y,
        }
    }
}

impl Display for Area {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "X: {}-{}, Y: {}-{}",
            self.pos_x,
            self.pos_x + self.size_x - 1,
            self.pos_y,
            self.pos_y + self.size_y - 1
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tile {
    pub amount: u64,
    pub area: Area,
}

impl Tile {
    pub fn has_treasures(&self, min_num: u64) -> bool {
        self.amount >= min_num
    }
    pub fn is_single_point(&self) -> bool {
        self.area.size_x == 1 && self.area.size_y == 1
    }
    pub fn split(&self) -> (Tile, Tile) {
        let size: u64 = self.area.size_x / 2;

        let left = Tile {
            amount: 0,
            area: Area::new(self.area.pos_x, self.area.pos_y, size, self.area.size_y),
        };
        let right = Tile {
            amount: 0,
            area: Area::new(
                self.area.pos_x + size,
                self.area.pos_y,
                self.area.size_x-size,
                self.area.size_y,
            ),
        };
        (left, right)
    }

    pub fn split_to_tiles(&self, preferred_tile_size: u64) -> Vec<Tile> {
        let mut result = Vec::new();
        let triples = self.area.size_x / preferred_tile_size;
        let singles = self.area.size_x % preferred_tile_size;
        let mut used_size = 0u64;

        for _ in 0..triples {
            result.push(Tile {
                amount: 0,
                area: Area {
                    pos_x: self.area.pos_x + used_size,
                    pos_y: self.area.pos_y,
                    size_x: preferred_tile_size,
                    size_y: self.area.size_y,
                },
            });
            used_size += preferred_tile_size;
        }

        for _ in 0..singles {
            result.push(Tile {
                amount: 0,
                area: Area {
                    pos_x: self.area.pos_x + used_size,
                    pos_y: self.area.pos_y,
                    size_x: 1,
                    size_y: self.area.size_y,
                },
            });
            used_size += 1;
        }
        result
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "amount: {},X: {}, Y: {}, sizeX: {},sizeY: {}",
            self.amount, self.area.pos_x, self.area.pos_y, self.area.size_x, self.area.size_y
        )
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Dig {
    pub depth: u64,
    #[serde(rename = "licenseID")]
    pub license_id: u64,
    #[serde(rename = "posX")]
    pub pos_x: u64,
    #[serde(rename = "posY")]
    pub pos_y: u64,
    #[serde(skip)]
    pub amount: u64,
}

impl Dig {
    pub fn from_tile(tile: Tile, license_id: u64) -> Dig {
        Dig {
            depth: 1,
            license_id,
            pos_x: tile.area.pos_x,
            pos_y: tile.area.pos_y,
            amount: tile.amount,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct License {
    #[serde(rename = "digAllowed")]
    pub dig_allowed: u64,
    #[serde(rename = "digUsed")]
    pub dig_used: u64,
    pub id: u64,
}

impl License {
    pub fn new() -> License {
        License {
            dig_allowed: 0,
            dig_used: 0,
            id: 0,
        }
    }
}

pub struct SplitMoneyList {
    pub money: MoneyList,
    pub exchange: MoneyList,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoneyList(Vec<u32>);

impl MoneyList {
    pub fn new() -> MoneyList {
        MoneyList { 0: vec![] }
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get_optimal_list(&self, max_cost: usize, min_cost: usize) -> SplitMoneyList {
        //1,6,11,21
        let l = self.len();
        let mut n;
        if l >= 21 {
            n = 21;
        } else if l >= 11 {
            n = 11;
        } else if l >= 6 {
            n = 6;
        } else {
            n = 1;
        }
        if n < min_cost {
            n = 0
        }
        if n > max_cost {
            n = max_cost
        }
        SplitMoneyList {
            money: self.0.iter().take(n).collect(),
            exchange: self.0.iter().rev().take(l - n).collect(),
        }
    }
}

impl<'a> FromIterator<&'a u32> for MoneyList {
    fn from_iter<T: IntoIterator<Item = &'a u32>>(iter: T) -> Self {
        let mut result = MoneyList::new();
        for e in iter {
            result.0.push(*e);
        }
        result
    }
}

#[derive(Serialize, Deserialize)]
pub struct TreasureList(pub Vec<String>);

impl TreasureList {
    pub fn new() -> TreasureList {
        TreasureList { 0: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Tile;

    #[test]
    fn test_split_to_tiles() {
        //(size_x of parent tile,preferred_tile_size,expected_tile_count)
        let test_data = [
            (17, 3, 7),
            (17, 1, 17),
            (17, 2, 9),
            (1, 1, 1),
            (3, 3, 1),
            (4, 2, 2),
            (4, 3, 2),
            (7, 3, 3),
        ];

        for t in test_data.iter() {
            check_tiles_in_vec(t.0, t.1, t.2)
        }
    }

    fn check_tiles_in_vec(size_x: u64, preferred_tile_size: u64, expected_tile_count: u64) {
        let parent_tile = Tile::new(0, 0, size_x, 1);
        let v = parent_tile.split_to_tiles(preferred_tile_size);
        let mut pos = 66666;
        let mut size_sum = 0;
        println!("test tile {} => ", parent_tile);

        assert_eq!(v.len(), expected_tile_count as usize);
        for t in v.iter() {
            print!("{}", *t);
            if pos == t.area.pos_x {
                assert!(false, "tiles with dublicate pos_x")
            }
            if t.area.size_x != preferred_tile_size && t.area.size_x != 1 {
                assert!(false, "")
            }
            pos = t.area.pos_x;
            size_sum += t.area.size_x;
            assert!(
                t.area.pos_x + t.area.size_x <= parent_tile.area.pos_x + parent_tile.area.size_x
            )
        }
        assert_eq!(size_sum, parent_tile.area.size_x)
    }
}
