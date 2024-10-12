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

pub struct QTrainer {
    pub model: LinerQNet,
    gamma: f32,
    optimizer: Optimizer,
}

impl QTrainer {
    pub fn new(vs: &VarStore, model: LinerQNet, lr: f64, gamma: f32) -> Self {
        let optimizer = Adam::default().build(&vs, lr).unwrap();
        Self {
            model,
            gamma,
            optimizer,
        }
    }

    fn tensor_train_step(
        &mut self,
        state: Tensor,
        action: Tensor,
        reward: Tensor,
        next_state: Tensor,
        done: Vec<bool>,
    ) {
        let pred = self.model.forward(&state);

        let mut target = pred.copy();
        for idx in 0..done.len() as i64 {
            let mut q_new = reward.i(idx);
            if !done[idx as usize] {
                q_new = q_new + self.gamma * self.model.forward(&next_state.i(idx)).max();
            }

            let _ = target.index_put_(
                &[
                    Some(Tensor::from_slice(&[idx])),
                    Some(Tensor::from_slice(&[action
                        .i(idx)
                        .argmax(0, false)
                        .int64_value(&[])])),
                ],
                &q_new,
                false,
            );
        }

        self.optimizer.zero_grad();
        let loss = target.mse_loss(&pred, tch::Reduction::Mean);
        loss.backward();

        self.optimizer.step();
    }

    pub fn train_multiple_steps(&mut self, snapshots: SnapshotConcat) {
        if !snapshots.is_built() {
            panic!(
                "SnapshotConcat is not built, filled: {:?}, target size: {:?}",
                snapshots.done.len(),
                snapshots.target_size
            );
        }
        let len = snapshots.target_size as i64;
        let state_view = [len, STATE_SIZE as i64];
        let next_state_view = [len, STATE_SIZE as i64];
        let action_view = [len, ACTION_SIZE as i64];
        let reward_view = [len, 1];

        let state = Tensor::from_slice(&snapshots.state).view(state_view);
        let next_state = Tensor::from_slice(&snapshots.next_state).view(next_state_view);
        let action = Tensor::from_slice(&snapshots.action).view(action_view);

        let reward = Tensor::from_slice(&snapshots.reward).view(reward_view);

        self.tensor_train_step(state, action, reward, next_state, snapshots.done);
    }

    pub fn train_single_step(&mut self, snapshot: &Snapshot) {
        let mut snapshots = SnapshotConcat::building(1);
        snapshots.push(snapshot);
        self.train_multiple_steps(snapshots);
    }
}
