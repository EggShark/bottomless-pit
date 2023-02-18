use std::collections::HashMap;

use crate::texture::Texture;

pub(crate) struct TextureCache {
    cache: HashMap<u32, ChachedTexture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn chache_update(&mut self) {
        self.cache.iter_mut().for_each(|(_, v)| v.time_since_used += 1);
        self.cahce_cleanup();
    }

    fn cahce_cleanup(&mut self) {
        let keys_to_remove = self.cache
            .iter()
            .filter_map(|(k, v)| (v.time_since_used > 60).then_some(*k))
            .collect::<Vec<u32>>();

        for key in keys_to_remove {
            self.cache.remove(&key);
        }
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureIndex {
        let chaced_texture = ChachedTexture {
            bind_group: texture.bind_group,
            time_since_used: 0
        };

        let index = TextureIndex {
            view: texture.view,
            sampler: texture.sampler,
            bind_group_layout: texture.bind_group_layout,
            id: texture.id,
        };

        self.cache.insert(texture.id, chaced_texture);

        index
    }
}

pub(crate) struct ChachedTexture {
    bind_group: wgpu::BindGroup,
    time_since_used: i32,
}

pub struct TextureIndex {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    // the info needed to recrate the texture when necciscarry
    pub(crate) id: u32 //crc32 checksum
}