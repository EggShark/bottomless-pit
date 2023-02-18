use std::collections::HashMap;

use crate::texture::Texture;

struct TextureCache {
    cache: HashMap<TextureIndex, ChachedTexture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
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
            id: 0,
        };

        //self.cache.insert();

        index
    }
}

struct ChachedTexture {
    bind_group: wgpu::BindGroup,
    time_since_used: i32,
}

struct TextureIndex {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    // the info needed to recrate the texture when necciscarry
    id: u32 //something to hash
}