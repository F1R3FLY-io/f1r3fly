use std::io::Cursor;

trait KeyValueStoreTrait {
    fn get<T, F>(&self, keys: Vec<Cursor<Vec<u8>>>, from_buffer: F) -> Vec<Option<T>>
    where
        F: Fn(Cursor<Vec<u8>>) -> T;

    fn put<T, F>(&self, kv_pairs: Vec<(Cursor<Vec<u8>>, T)>, to_buffer: F) -> ()
    where
        F: Fn(T) -> Cursor<Vec<u8>>;

    fn delete(&self, keys: Vec<Cursor<Vec<u8>>>) -> i32;

    fn iterate<T, F>(&self, f: F) -> T
    where
        F: FnOnce(Box<dyn Iterator<Item = (Cursor<Vec<u8>>, Cursor<Vec<u8>>)>>) -> T;
}

pub struct KeyValueStore {}
