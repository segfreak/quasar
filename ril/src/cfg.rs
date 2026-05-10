use crate::ir::*;

pub struct CFG {
    pub blocks: Vec<Block>,
}

impl CFG {
    pub fn block_iter(&self) -> impl DoubleEndedIterator<Item = &Block> {
        self.blocks.iter()
    }

    pub fn block(&self, i: BlockId) -> &Block {
        &self.blocks[i as usize]
    }
}
