use std::{collections::HashMap, ops::Index};

#[derive(Debug)]
pub(crate) struct ResourceCache<T> {
    resources: HashMap<u32, CachedResource<T>>
}

#[derive(Debug)]
pub(crate) struct CachedResource<T> {
    pub(crate) resource: T,
    pub(crate) time_since_used: u32,
}

impl<T> ResourceCache<T> {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn cache_update(&mut self) {
        self.resources
            .iter_mut()
            .for_each(|(_, v)| v.time_since_used += 1);
        self.cahce_cleanup();
    }

    fn cahce_cleanup(&mut self) {
        let keys_to_remove = self
            .resources
            .iter()
            .filter_map(|(k, v)| (v.time_since_used > 60).then_some(*k))
            .collect::<Vec<u32>>();

        for key in keys_to_remove {
            self.resources.remove(&key);
        }
    }

    pub fn add_item(&mut self, resource: T, key: u32) {
        let cache_item = CachedResource {
            resource,
            time_since_used: 0
        };

        self.resources.insert(key, cache_item);
    }

    pub fn get(&self, key: u32) -> Option<&CachedResource<T>> {
        self.resources.get(&key)
    }

    pub fn get_mut(&mut self, key: u32) -> Option<&mut CachedResource<T>> {
        self.resources.get_mut(&key)
    }
}

impl<T> Index<u32> for ResourceCache<T> {
    type Output = CachedResource<T>;
    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).unwrap()
    }
}