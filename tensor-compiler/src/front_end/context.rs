use std::{ cell::RefCell, rc::Rc };
use getset::{ CopyGetters, Getters, MutGetters, Setters };
use hashbrown::HashMap;
use serde::Serialize;
use tensor_common::{
    block_info::BlockInfo,
    block_manager::{ BlockManager, BlockType },
    shape::Shape,
};
use tensor_traits::tensor::CommonBounds;
use tensor_types::{ convertion::Convertor, dtype::Dtype };

use super::{ _tensor::_Tensor, control_flow::{ ControlFlowState, If }, tensor::Tensor };

#[derive(Clone, Getters, Setters, MutGetters, CopyGetters)]
pub struct Context {
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    ctx: Rc<RefCell<_Context>>,
}

impl Context {
    pub fn new() -> Self {
        let ctx = Rc::new(
            RefCell::new(_Context {
                saved_blocks: HashMap::new(),
                blocks_manager: BlockManager::new(),
                block_stack: vec![BlockInfo::new(0, 0)],
                block_id: 0,
                acc_node_id: 0,
                nodes: HashMap::new(),
            })
        );
        Self { ctx }
    }

    pub fn push_stack(&self, name: &str, block_type: BlockType) {
        self.ctx.borrow_mut().push_stack(name, block_type);
    }

    pub fn arange<T: Convertor + CommonBounds>(&self, start: T, end: T, step: T) -> Tensor {
        let ret = Tensor::arange(start, end, step, self.ctx.clone());
        ret
    }

    pub fn randn<S: Into<Shape>>(&self, shape: S, mean: f64, std: f64, dtype: Dtype) -> Tensor {
        let shape = shape.into();
        let ret = Tensor::randn(self.ctx.clone(), mean, std, shape, dtype);
        ret
    }

    pub fn r#if(&self, cond: Tensor) -> If<Tensor> {
        If::new(cond, self)
    }

    pub fn cond<F>(&self, f: F) -> Tensor where F: FnOnce() -> Tensor {
        self.push_stack("cond", BlockType::Function);
        let ret = f();
        self.pop_stack();
        ret
    }
}

#[derive(Clone, Getters, Setters, MutGetters, CopyGetters, Serialize)]
pub struct _Context {
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    saved_blocks: HashMap<usize, usize>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    blocks_manager: BlockManager,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    block_stack: Vec<BlockInfo>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    block_id: usize,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    acc_node_id: usize,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    nodes: HashMap<usize, _Tensor>,
}

impl _Context {
    pub fn push_stack(&mut self, name: &str, block_type: BlockType) {
        self.block_id += 1;
        let parent_block_id = self.block_stack.last().unwrap().current_id();
        self.block_stack.push(BlockInfo::new(*self.block_id(), parent_block_id));
        self.saved_blocks.insert(*self.block_id(), parent_block_id);
        self.blocks_manager.insert_parent(
            *self.block_id(),
            name.to_string(),
            parent_block_id,
            block_type
        );
    }

    pub fn pop_stack(&mut self) {
        self.block_stack.pop();
    }

    pub fn increment_id(&mut self) {
        self.acc_node_id += 1;
    }
}

impl ControlFlowState for Context {
    fn block_id(&self) -> usize {
        *self.ctx.borrow().block_id()
    }
    fn pop_stack(&self) {
        self.ctx.borrow_mut().pop_stack();
    }
    fn push_stack(&self, name: &str, block_type: BlockType) {
        self.ctx.borrow_mut().push_stack(name, block_type);
    }
}
