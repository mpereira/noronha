use uuid::Uuid;

pub type NodeId = Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct UnknownNode {
    pub address: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Node {
    pub id: NodeId,
    pub address: String,
    pub name: String,
}
