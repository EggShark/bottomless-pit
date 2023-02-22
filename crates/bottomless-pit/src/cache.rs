use std::collections::HashMap;

use crate::texture::Texture;

#[derive(Debug)]
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
            id: texture.id,
        };

        self.cache.insert(texture.id, chaced_texture);

        index
    }

    pub fn rebuild_from_index(&mut self, index: &TextureIndex, device: &wgpu::Device) {
        let bind_group_layout = Texture::make_bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&index.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&index.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let chaced_texture = ChachedTexture {
            bind_group: bind_group,
            time_since_used: 0
        };

        self.cache.insert(index.id, chaced_texture);
    }

    pub fn get(&self, key: &TextureIndex) -> Option<&ChachedTexture> {
        self.cache.get(&key.id)
    }

    pub fn get_mut(&mut self, key: &TextureIndex) -> Option<&mut ChachedTexture> {
        self.cache.get_mut(&key.id)
    }
}

impl std::ops::Index<TextureIndex> for TextureCache {
    type Output = ChachedTexture;
    fn index(&self, index: TextureIndex) -> &Self::Output {
        self.cache.get(&index.id).unwrap_or_else(|| panic!("No Texture found for id {}", index.id))
    }
}

impl std::ops::Index<u32> for TextureCache {
    type Output = ChachedTexture;
    fn index(&self, index: u32) -> &Self::Output {
        self.cache.get(&index).unwrap_or_else(|| panic!("No Texture found for id {}", index))
    }
}

#[derive(Debug)]
pub(crate) struct ChachedTexture {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) time_since_used: i32,
}

pub struct TextureIndex {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    // the info needed to recrate the texture when necciscarry
    pub(crate) id: u32 //crc32 checksum
}