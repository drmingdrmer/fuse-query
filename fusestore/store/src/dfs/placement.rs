use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::fs::IFileSystem;

pub type NodeId = i64;

// A slot is a virtual and intermediate allocation unit in a distributed storage.
// The key of an object is mapped to a slot by hashing. A slot is assigned to several physical servers(normally 3 for durability).
pub struct Slot {
    node_ids: Vec<i64>,
}

pub struct Cluster {
    slots: Vec<Slot>,
    nodes: HashMap<NodeId, Node>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub address: String,
}

impl Cluster {
    pub fn key_to_nodes(&self, key: &String) -> Vec<Node> {
        // TODO use consistent hash if need to extend cluster.
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hsh = hasher.finish();
        let slot_idx = hsh % self.slots.len() as u64;
        let slot = &self.slots[slot_idx as usize];

        slot.node_ids
            .iter()
            .map(|nid| (*self.nodes.get(nid).unwrap()).clone())
            .collect()
    }
}

pub struct DFS {
    pub cluster: Arc<Mutex<Cluster>>,
}

#[async_trait]
impl IFileSystem for DFS {
    async fn add<'a>(
        &'a self,
        path: impl AsRef<std::path::Path> + Send + 'a,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let key = path.as_ref().to_str().unwrap().to_string();

        let c = self.cluster.lock().await;
        let nodes = c.key_to_nodes(&key);

        let (tx, rx) = tokio::mpsc::channel(5);
        for node in nodes.iter() {
            tokio::spwan(async move {
                let cli = store_client::new();
                let rst = cli.send(data).await;
                tx.send(rst).await
            });
        }

        for x in rx.iter() {
            match x {
                Ok(_) => {
                    //
                }
                Err(err) => {
                    //
                }
            }
            if n >= quorum {
                // ok
                break;
            }
        }

        self.meta_add(key).await
    }

    async fn read_all<'a>(
        &'a self,
        path: impl AsRef<std::path::Path> + Send + 'a,
    ) -> anyhow::Result<Vec<u8>> {
        todo!()
    }

    async fn list<'a>(
        &'a self,
        path: impl AsRef<std::path::Path> + Send + 'a,
    ) -> anyhow::Result<crate::fs::ListResult> {
        todo!()
    }
}
