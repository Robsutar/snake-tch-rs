use std::path::Path;

use crate::{
    game::{GridPos, Scene, SnakeOrientation},
    model::{LinerQNet, QTrainer, Snapshot, SnapshotConcat, ACTION_SIZE, STATE_SIZE},
    utils::FixedVecDeque,
    DType, DEVICE,
};

use rand::{thread_rng, Rng};
use tch::{
    nn::{Module, VarStore},
    Tensor,
};

const MAX_MEMORY: usize = 100000;
const BATCH_SIZE: usize = 1000;
const LR: f64 = 0.001;

pub struct Agent {
    pub n_games: usize,
    memory: FixedVecDeque<Snapshot>,
    trainer: QTrainer,
    vs: VarStore,
}

impl std::fmt::Debug for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agent")
            .field("number of games", &self.n_games)
            .field("memory size", &self.memory.len())
            .finish()
    }
}

impl Agent {
    pub fn load_if_exists(file_name: &str) -> Self {
        let vs = VarStore::new(DEVICE);

        let mut exit = Self {
            n_games: 0,
            memory: FixedVecDeque::new(MAX_MEMORY),
            trainer: QTrainer::new(
                &vs,
                LinerQNet::new(&vs, STATE_SIZE as i64, 256, ACTION_SIZE as i64),
                LR,
                0.9,
            ),
            vs,
        };

        let file_name = Path::new("./model").join(file_name);
        if file_name.exists() {
            exit.vs.load(file_name).unwrap();
        }

        exit
    }

    pub fn save(&self, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let model_folder_path = Path::new("./model");
        if !model_folder_path.exists() {
            std::fs::create_dir(model_folder_path)?;
        }

        let file_name = model_folder_path.join(file_name);
        Ok(self.vs.save(file_name)?)
    }

    pub fn get_state(scene: &Scene) -> [DType; STATE_SIZE] {
        let head_pos = &scene.snake_head.ge.pos;
        let food_pos = &scene.apple.ge.pos;

        let point_l = GridPos::new(head_pos.x - 1, head_pos.y);
        let point_r = GridPos::new(head_pos.x + 1, head_pos.y);
        let point_u = GridPos::new(head_pos.x, head_pos.y - 1);
        let point_d = GridPos::new(head_pos.x, head_pos.y + 1);

        let head_direction = &scene.snake_head.orientation;
        let dir_l = head_direction == &SnakeOrientation::Left;
        let dir_r = head_direction == &SnakeOrientation::Right;
        let dir_u = head_direction == &SnakeOrientation::Up;
        let dir_d = head_direction == &SnakeOrientation::Down;

        [
            // Danger straight
            ((dir_r & scene.is_collision(&point_r))
                | (dir_l & scene.is_collision(&point_l))
                | (dir_u & scene.is_collision(&point_u))
                | (dir_d & scene.is_collision(&point_d))) as u8 as DType,
            // Danger right
            ((dir_u & scene.is_collision(&point_r))
                | (dir_d & scene.is_collision(&point_l))
                | (dir_l & scene.is_collision(&point_u))
                | (dir_r & scene.is_collision(&point_d))) as u8 as DType,
            // Danger left
            ((dir_d & scene.is_collision(&point_r))
                | (dir_u & scene.is_collision(&point_l))
                | (dir_r & scene.is_collision(&point_u))
                | (dir_l & scene.is_collision(&point_d))) as u8 as DType,
            // Move head_direction
            dir_l as u8 as DType,
            dir_r as u8 as DType,
            dir_u as u8 as DType,
            dir_d as u8 as DType,
            // Food location
            (food_pos.x < head_pos.x) as u8 as DType, // food left
            (food_pos.x > head_pos.x) as u8 as DType, // food right
            (food_pos.y < head_pos.y) as u8 as DType, // food up
            (food_pos.y > head_pos.y) as u8 as DType, // food down
        ]
    }

    pub fn remember(&mut self, snapshot: Snapshot) {
        self.memory.push(snapshot);
    }

    pub fn train_long_memory(&mut self) {
        let mini_sample = if self.memory.len() > BATCH_SIZE {
            let mut mini_sample = SnapshotConcat::building(BATCH_SIZE);
            let mut rng = thread_rng();
            for index in
                rand::seq::index::sample(&mut rng, self.memory.len(), BATCH_SIZE).into_iter()
            {
                mini_sample.push(&self.memory.as_deque()[index]);
            }

            mini_sample
        } else {
            let mut mini_sample = SnapshotConcat::building(self.memory.len());
            for snapshot in self.memory.as_deque() {
                mini_sample.push(snapshot);
            }

            mini_sample
        };

        self.trainer.train_multiple_steps(mini_sample);
    }

    pub fn train_with_last(&mut self, count: usize) {
        if self.memory.len() < count {
            panic!("There are no enough examples.");
        }
        let mut mini_sample = SnapshotConcat::building(count);
        for index in self.memory.len() - count..self.memory.len() {
            mini_sample.push(&self.memory.as_deque()[index]);
        }
        self.trainer.train_multiple_steps(mini_sample);
    }

    pub fn get_action(&self, state: &[DType; STATE_SIZE]) -> [DType; ACTION_SIZE] {
        let mut rng = thread_rng();

        let epsilon: i32 = 80 - self.n_games as i32;
        let mut final_move = [0.0, 0.0, 0.0];
        if rng.gen_range(0..200) < epsilon {
            let target_move = rng.gen_range(0..2);
            final_move[target_move] = 1.0;
        } else {
            let state0 = Tensor::from_slice(state);
            let prediction = self.trainer.model.forward(&state0);
            let target_mode = prediction.argmax(0, false).int64_value(&[]);
            final_move[target_mode as usize] = 1.0;
        }

        return final_move;
    }
}
