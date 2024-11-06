use std::slice::IterMut;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use zkwasm_rest_abi::StorageData;
use zkwasm_rust_sdk::{require, wasm_dbg};

//基础玩法，没有实现抽卡(随机数）、阶梯化价格（modifier）、成长（时间序列）、收货金币（时间序列）
#[derive(Serialize, Deserialize, Debug, PartialEq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
enum Command {
    BuyDolphin = 0,
    BuyFood = 1,
    BuyMedicine = 2,
    FeedDolphin = 3,
    HealDolphin = 4,
    AttackEvilWhale = 5,
    BuyPopulationNumber = 6,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
enum LifeStage {
    Baby = 0,
    Adult = 1
}

#[derive(Serialize, Deserialize, Debug, PartialEq, TryFromPrimitive, Copy, Clone)]
#[repr(u32)]
enum DolphinName {
    DolphinArcher = 0,
    DolphinKnight = 1,
    DolphinMage = 2,
    DolphinPriest = 3,
    DolphinRogue = 4,
    DolphinWarrior = 5,
    DolphinWizard = 6,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Dolphin {
    id: u64,
    name: DolphinName,
    level: u64,
    life_stage: LifeStage,
    join_time: u64,
    health: u64,
    satiety: u64,
    generated_coins: u64,
    collected_coins: u64,
}


impl StorageData for Dolphin {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        Dolphin {
           id: *u64data.next().unwrap(),
           name: DolphinName::try_from(*u64data.next().unwrap() as u32).unwrap(),
           level: *u64data.next().unwrap(),
           life_stage: LifeStage::try_from(*u64data.next().unwrap() as u32).unwrap(),
           join_time: *u64data.next().unwrap(),
           health: *u64data.next().unwrap(),
           satiety: *u64data.next().unwrap(),
           generated_coins:*u64data.next().unwrap(),
           collected_coins: *u64data.next().unwrap(),
        }
    }

    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.id);
        data.push(self.name as u64);
        data.push(self.level);
        data.push(self.life_stage as u64);
        data.push(self.join_time);
        data.push(self.health);
        data.push(self.satiety);
        data.push(self.generated_coins);
        data.push(self.collected_coins);
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct PlayerData {
    pid: [u64; 2],
    coins_balance: u64,
    food_number: u64,
    medicine_number: u64,
    population_number: u64,
    dolphins: Vec<Dolphin>,
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let mut player_data = PlayerData {
            pid: [
                *u64data.next().unwrap(),
                *u64data.next().unwrap(),
            ],
            coins_balance: *u64data.next().unwrap(),
            food_number: *u64data.next().unwrap(),
            medicine_number: *u64data.next().unwrap(),
            population_number: *u64data.next().unwrap(),
            dolphins: vec![]
        };
        let dsize = *u64data.next().unwrap() as usize;
        for _ in 0..dsize {
            player_data.dolphins.push(Dolphin::from_data(u64data));
        }
        player_data
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.pid[0]);
        data.push(self.pid[1]);
        data.push(self.coins_balance);
        data.push(self.food_number);
        data.push(self.medicine_number);
        data.push(self.population_number);
        data.push(self.dolphins.len() as u64);
        for i in 0..self.dolphins.len() {
            self.dolphins[i].to_data(data);
        }
    }
}


// update state via command, Command：购买海豚，购买食物，购买药品，喂食海豚[I]，喂药海豚[I], 迎击邪恶巨鲸，购买栏位(population_number)
pub fn update_state(command: u8, player: &mut PlayerData, dolphin_id: u64) -> Result <(), u32> {
    Command::try_from(command as u32).map_or(
        Err(1),
        |command| {
            match command {
                Command::BuyDolphin => {
                    unsafe {
                        require(player.coins_balance >= 100);
                        require(!(player.dolphins.len() as u64 >= player.population_number));
                    }
                    player.coins_balance -= 100;
                    player.dolphins.push(Dolphin {
                        id: player.dolphins.len() as u64,
                        name: DolphinName::DolphinArcher,
                        level: 1,
                        life_stage: LifeStage::Baby,
                        join_time: 0,
                        satiety: 100,
                        health: 100,
                        generated_coins: 0,
                        collected_coins: 0,
                    });
                }
                Command::BuyFood => {
                    unsafe {
                        require(player.coins_balance >= 10);
                    }
                    player.coins_balance -= 10;
                    player.food_number += 1;
                }
                Command::BuyMedicine => {
                    unsafe {
                        require(player.coins_balance >= 20);
                    }
                    player.coins_balance -= 20;
                    player.medicine_number += 1;
                }
                Command::FeedDolphin => {
                    unsafe {
                        require(player.food_number > 0);
                        require(player.dolphins.len() > 0);
                    }
                    player.food_number -= 1;
                    player.dolphins[dolphin_id as usize].satiety += 10;
                }
                Command::HealDolphin => {
                    unsafe {
                        require(player.medicine_number > 0);
                        require(player.dolphins.len() > 0);
                    }
                    player.medicine_number -= 1;
                    player.dolphins[dolphin_id as usize].health = 100;
                }
                Command::AttackEvilWhale => {
                    unsafe {
                        require(player.dolphins.len() > 0);
                    }
                    //all dolphins health to 0
                    for dolphin in player.dolphins.iter_mut() {
                        dolphin.health = 0;
                    }
                    //collect all coins
                    for dolphin in player.dolphins.iter_mut() {
                        player.coins_balance += dolphin.generated_coins;
                        dolphin.generated_coins = 0;
                        dolphin.collected_coins += dolphin.generated_coins;
                    }
                }
                Command::BuyPopulationNumber => {
                    unsafe {
                        require(player.coins_balance >= 100);
                    }
                    player.coins_balance -= 200;
                    player.population_number += 1;
                }
            };
            Ok(())
        })
}
