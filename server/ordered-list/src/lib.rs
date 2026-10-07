use std::marker::PhantomData;

pub struct OrderedList<T, K, F>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    items: Vec<T>,
    key: F,
    _marker: PhantomData<K>,
}

impl<T, K, F> OrderedList<T, K, F>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    pub fn new_by(key: F) -> Self {
        Self {
            items: Vec::new(),
            key,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push(&mut self, value: T) -> u32 {
        let key_fn = &self.key;
        let key = key_fn(&value);
        let index = self
            .items
            .partition_point(|existing| key_fn(existing) <= key);
        self.items.insert(index, value);
        index as u32
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        self.items.get(index as usize)
    }

    pub fn remove(&mut self, index: u32) -> T {
        self.items.remove(index as usize)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.items.iter()
    }
}

impl<T, K, F> IntoIterator for OrderedList<T, K, F>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::OrderedList;

    #[derive(Debug, PartialEq, Eq)]
    struct Message {
        command: String,
        timestamp: u64,
    }

    #[test]
    fn orders_messages_by_timestamp() {
        let mut list = OrderedList::new_by(|message: &Message| message.timestamp);

        list.push(Message {
            command: "later".into(),
            timestamp: 20,
        });
        list.push(Message {
            command: "first".into(),
            timestamp: 10,
        });
        list.push(Message {
            command: "middle".into(),
            timestamp: 15,
        });

        let commands = list
            .iter()
            .map(|message| message.command.as_str())
            .collect::<Vec<_>>();

        assert_eq!(commands, vec!["first", "middle", "later"]);
    }

    #[test]
    fn keeps_equal_keys_in_insertion_order() {
        let mut list = OrderedList::new_by(|message: &Message| message.timestamp);

        list.push(Message {
            command: "a".into(),
            timestamp: 5,
        });
        list.push(Message {
            command: "b".into(),
            timestamp: 5,
        });
        list.push(Message {
            command: "c".into(),
            timestamp: 5,
        });

        let commands = list
            .iter()
            .map(|message| message.command.as_str())
            .collect::<Vec<_>>();

        assert_eq!(commands, vec!["a", "b", "c"]);
    }
}
