use tch::nn::{self, Adam, Linear, Module, Optimizer, OptimizerConfig, VarStore};
use tch::{IndexOp, Tensor};

use crate::DType;

pub const STATE_SIZE: usize = 11;
pub const ACTION_SIZE: usize = 3;
pub type REWARD = DType;

pub struct Snapshot {
    pub state: [DType; STATE_SIZE],
    pub action: [DType; ACTION_SIZE],
    pub reward: REWARD,
    pub next_state: [DType; STATE_SIZE],
    pub done: bool,
}
pub struct SnapshotConcat {
    pub state: Vec<DType>,
    pub action: Vec<DType>,
    pub reward: Vec<DType>,
    pub next_state: Vec<DType>,
    pub done: Vec<bool>,
    target_size: usize,
}
impl SnapshotConcat {
    pub fn building(target_size: usize) -> Self {
        Self {
            state: Vec::with_capacity(target_size * STATE_SIZE),
            action: Vec::with_capacity(target_size * ACTION_SIZE),
            reward: Vec::with_capacity(target_size),
            next_state: Vec::with_capacity(target_size * STATE_SIZE),
            done: Vec::with_capacity(target_size),
            target_size,
        }
    }

    pub fn push(&mut self, snapshot: &Snapshot) {
        if self.is_built() {
            panic!("SnapshotConcat already built (full).");
        }
        self.state.extend_from_slice(&snapshot.state);
        self.action.extend_from_slice(&snapshot.action);
        self.reward.push(snapshot.reward);
        self.next_state.extend_from_slice(&snapshot.next_state);
        self.done.push(snapshot.done);
    }

    pub fn is_built(&self) -> bool {
        self.done.len() == self.target_size
    }
}

#[derive(Debug)]
pub struct LinerQNet {
    fc1: Linear,
    fc2: Linear,
}
impl LinerQNet {
    pub fn new(vs: &VarStore, input_size: i64, hidden_size: i64, output_size: i64) -> Self {
        let fc1 = nn::linear(&vs.root(), input_size, hidden_size, Default::default());
        let fc2 = nn::linear(&vs.root(), hidden_size, output_size, Default::default());

        Self { fc1, fc2 }
    }
}
impl Module for LinerQNet {
    fn forward(&self, xs: &Tensor) -> Tensor {
        xs.apply(&self.fc1).relu().apply(&self.fc2)
    }
}

