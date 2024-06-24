use std::{ cell::RefCell, rc::Rc };
use getset::{ CopyGetters, Getters, MutGetters, Setters };
use hashbrown::HashMap;
use tensor_common::{ block_info::BlockInfo, block_manager::{ BlockManager, BlockType } };

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters)]
pub struct Context {
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    ctx: Rc<RefCell<_Context>>,
}

impl Context {
    pub fn push_stack(&self, name: &str, block_type: BlockType) {
        self.ctx.borrow_mut().push_stack(name, block_type);
    }
}

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters)]
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
