#![warn(clippy::pedantic)]

use generic_array::GenericArray;
use serde::Serialize;
use sha3::{Digest, Sha3_256};

use std::convert::From;

mod peering;

#[derive(Serialize, Debug, Copy, Clone)]
struct Block {
    // header
    prev_hash: [u8; 32],
    height: usize,
    nonce: u32,
}

#[derive(Serialize, Debug)]
struct BlockChain {
    blocks: Vec<Block>,
    unconfirmed: Vec<Vec<Block>>,
}

impl BlockChain {
    pub fn check_hashes(&self) -> bool {
        for (i, b) in self.blocks.iter().enumerate() {
            if i != 0
                && GenericArray::from(b.prev_hash)
                    != Sha3_256::digest(&bincode::serialize(&self.blocks[i - 1]).unwrap())
            {
                return false;
            }
        }
        true
    }
    pub fn new() -> BlockChain {
        BlockChain {
            blocks: Vec::new(),
            unconfirmed: Vec::new(),
        }
    }
    pub fn check_height(&self) -> bool {
        for (i, b) in self.blocks.iter().enumerate() {
            if i != b.height {
                return false;
            }
        }
        true
    }
    pub fn check(&self) -> bool {
        self.check_hashes() && self.check_height()
    }

    // Note: Run .check() first!
    pub fn prune_unconfirmed(&mut self) {
        if !self.unconfirmed.is_empty() {
            let mut remove = Vec::new();
            for (i, chain) in self.unconfirmed.iter().enumerate() {
                if self.blocks.len() != chain[0].height {
                    remove.push(i);
                    continue;
                }
                for (x, b) in chain.iter().enumerate() {
                    if b.height != x + self.blocks.len() {
                        remove.push(i);
                        continue;
                    }
                }
                for (x, b) in chain.iter().enumerate() {
                    if x == 0
                        && GenericArray::from(b.prev_hash)
                            != Sha3_256::digest(
                                &bincode::serialize(&self.blocks[self.blocks.len() - 1]).unwrap(),
                            )
                    {
                        remove.push(i);
                        continue;
                    }
                    if x != 0
                        && GenericArray::from(b.prev_hash)
                            != Sha3_256::digest(&bincode::serialize(&chain[x - 1]).unwrap())
                    {
                        remove.push(i);
                        continue;
                    }
                }
            }
            for r in remove {
                self.unconfirmed.remove(r);
            }
        }
    }
    // Run .prune_unconfirmed() after!
    pub fn add_unconfirmed(&mut self, un: Vec<Block>) {
        self.unconfirmed.push(un);
    }
}

impl Block {
    pub fn mine(self, difficulty: u8) -> u32 {
        let mut block = self;
        while !(check_mine(
            From::from(Sha3_256::digest(&bincode::serialize(&block).unwrap())),
            difficulty,
        )) {
            block.nonce += 1;
        }
        block.nonce
    }
}

fn check_mine(hash: [u8; 32], difficulty: u8) -> bool {
    for i in 0..difficulty * 2 {
        if hash[i as usize] != 0 {
            return false;
        }
    }
    true
}

impl From<Vec<Block>> for BlockChain {
    fn from(v: Vec<Block>) -> BlockChain {
        BlockChain {
            blocks: v,
            unconfirmed: Vec::new(),
        }
    }
}

fn main() {
    {
        let mut b = BlockChain::from(vec![
            Block {
                prev_hash: [0; 32],
                height: 0,
                nonce: 0,
            },
            Block {
                prev_hash: From::from(Sha3_256::digest(
                    &bincode::serialize(&Block {
                        prev_hash: [0; 32],
                        height: 0,
                        nonce: 0,
                    })
                    .unwrap(),
                )),
                height: 1,
                nonce: 0,
            },
        ]);
        assert!(b.check());
        b.add_unconfirmed(vec![Block {
            prev_hash: From::from(Sha3_256::digest(&bincode::serialize(&b.blocks[1]).unwrap())),
            height: 2,
            nonce: 0,
        }]);
        b.prune_unconfirmed();
    }
}
