use super::Ctx;
use vg::renderer::*;

pub struct Tex {
    size: Size,
    pub tex: webgl_stdweb::WebGLTexture,
}

impl vg::renderer::Texture<super::Gfx> for Tex {
    fn new(r: &mut super::Gfx, size: &Size, color: &Color) -> Self {
        Self::from_data(r, size, &vec![*color; size[0] * size[1]])
    }

    fn from_data(r: &mut super::Gfx, size: &Size, data: &Vec<Color>) -> Self {
        let mut buffer = Vec::with_capacity(data.len() * 4);
        for d in data {
            buffer.push((d[0] * 255.0) as u8);
            buffer.push((d[1] * 255.0) as u8);
            buffer.push((d[2] * 255.0) as u8);
            buffer.push((d[3] * 255.0) as u8);
        }
        
        let ctx = &r.surface.ctx;
        let tex = ctx.create_texture();
        ctx.bind_texture(Ctx::TEXTURE_2D, tex.as_ref());
        // Note, the third arg (internalFormat) _should_ be glenum but is
        // mistakenly marked as a glint in webgl_stdlib; should I issue about it?
        ctx.tex_image2_d(
            Ctx::TEXTURE_2D,
            0,
            Ctx::RGBA as _,
            size[0] as _,
            size[1] as _,
            0,
            Ctx::RGBA,
            Ctx::UNSIGNED_BYTE,
            Some(buffer.as_slice()),
        );
        // TODO: Add mipmap option?
        // ctx.generate_mipmap(Ctx::TEXTURE_2D);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MAG_FILTER, Ctx::NEAREST as _);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MIN_FILTER, Ctx::NEAREST as _);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_S, Ctx::CLAMP_TO_EDGE as _);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_T, Ctx::CLAMP_TO_EDGE as _);

        let tex = tex.unwrap();
        Tex { tex, size: *size }
    }

    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl vg::renderer::Target<super::Gfx> for Tex {
    fn size(&self) -> Size {
        self.size
    }

    fn set(&mut self, color: &Color) {
        unimplemented!()
    }

    fn draw(&mut self, texture: &Self, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}