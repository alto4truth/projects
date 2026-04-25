#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: usize,
    pub payload: String,
    pub previous_hash: String,
}

impl Block {
    pub fn new(index: usize, payload: impl Into<String>, previous_hash: impl Into<String>) -> Self {
        Self {
            index,
            payload: payload.into(),
            previous_hash: previous_hash.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::new(0, "genesis", "0")],
        }
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn latest(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn add_block(&mut self, payload: impl Into<String>) -> &Block {
        let previous_hash = self
            .latest()
            .map(|block| format!("{}:{}", block.index, block.payload))
            .unwrap_or_else(|| "0".to_string());
        let next_index = self.blocks.len();
        self.blocks
            .push(Block::new(next_index, payload, previous_hash));
        self.blocks
            .last()
            .expect("blockchain contains the newly inserted block")
    }

    pub fn iter(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_genesis_block() {
        let chain = Blockchain::new();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.latest().map(|block| block.payload.as_str()), Some("genesis"));
    }

    #[test]
    fn appends_blocks() {
        let mut chain = Blockchain::new();
        let block = chain.add_block("payload");
        assert_eq!(block.index, 1);
        assert_eq!(chain.len(), 2);
    }
}
