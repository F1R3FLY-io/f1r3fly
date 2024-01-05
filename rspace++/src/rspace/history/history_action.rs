// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryAction.scala
type KeyPath = Vec<u8>;

pub enum HistoryAction {
    Insert(InsertAction),
    Delete(DeleteAction),
}

trait HistoryActionTrait {
    fn key(&self) -> &KeyPath;
}

struct InsertAction {
    key: KeyPath,
    hash: blake3::Hash,
}

impl HistoryActionTrait for InsertAction {
    fn key(&self) -> &KeyPath {
        &self.key
    }
}

struct DeleteAction {
    key: KeyPath,
}

impl HistoryActionTrait for DeleteAction {
    fn key(&self) -> &KeyPath {
        &self.key
    }
}
