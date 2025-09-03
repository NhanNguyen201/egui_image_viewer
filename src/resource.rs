use std::hash::{Hash, Hasher};
use egui::{Pos2, TextureHandle, TextureId, Vec2};
use image::DynamicImage;
use std::path::PathBuf;

pub trait FitIn {
    fn is_fit_in(&self, container: egui::Rect) -> bool;
}

impl FitIn for egui::Rect {
    fn is_fit_in(&self, container: egui::Rect) -> bool {
        !(self.max.x < container.min.x) && !(self.max.y < container.min.y) && !{self.min.x > container.max.x} && !(self.min.y > container.max.y)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct CropRect {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32
}


impl Hash for CropRect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.top.to_bits().hash(state);
        self.left.to_bits().hash(state);
        self.bottom.to_bits().hash(state);
        self.right.to_bits().hash(state);
    }
}

impl Default for CropRect {
    fn default() -> Self {
        Self {
            top: 0.,
            left: 0.,
            bottom: 0.,
            right: 0.
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ImageTranforms {
    pub scale: f32,
    pub original_scale: f32,
    pub pos: Pos2,
    pub rotation: f32,
    pub opacity: f32,
    pub croped: Option<CropRect>,
    pub size: Vec2
}

impl Hash for ImageTranforms {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert f32 to bits for hashing since f32 doesn't implement Hash
        self.scale.to_bits().hash(state);
        self.original_scale.to_bits().hash(state);
        self.pos.x.to_bits().hash(state);
        self.pos.y.to_bits().hash(state);
        self.rotation.to_bits().hash(state);
        self.opacity.to_bits().hash(state);
        self.size.x.to_bits().hash(state);
        self.size.y.to_bits().hash(state);
        self.croped.hash(state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageProps {
    pub uv_rect: egui::Rect
}

impl Hash for ImageProps {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uv_rect.min.x.to_bits().hash(state);
        self.uv_rect.min.y.to_bits().hash(state);
        self.uv_rect.max.x.to_bits().hash(state);
        self.uv_rect.max.y.to_bits().hash(state);
    }
}

impl Default for ImageProps {
    fn default() -> Self {
        Self { uv_rect: egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)) }
    }
}

impl Default for ImageTranforms {
    fn default() -> Self {
        Self { scale: 1., original_scale: 1.,pos: Pos2::ZERO, rotation: 0.0, opacity: 1.0, croped: None, size: Vec2::ZERO }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActiveImage {
    pub id: TextureId,
}
impl ActiveImage {
    pub fn new(id: TextureId) -> Self {
        Self { id }
    }

    pub fn get_texture_id(&self) -> TextureId {
        self.id
    }
}

#[derive(Clone, PartialEq)]
pub struct OutputTexture {
    pub image: Option<DynamicImage>,
    pub original_scale: f32,
    pub texture_handle: Option<TextureHandle>,
    pub file_path: Option<PathBuf>,
    pub image_ratio: f32,
    pub is_active: bool,
    pub transform: ImageTranforms,
    pub image_props: ImageProps
}

impl Hash for OutputTexture {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash file_path if it exists (skip the image and texture_handle as they're not hashable)
        self.file_path.as_ref().map(|p| p.hash(state));
        self.original_scale.to_bits().hash(state);
        self.image_ratio.to_bits().hash(state);
        self.is_active.hash(state);
        self.transform.hash(state);
        self.image_props.hash(state);
    }
}



impl Default for OutputTexture {
    fn default() -> Self {
        Self {
            image: None,
            texture_handle: None,
            file_path: None,
            image_ratio: 1.,
            original_scale: 1.,
            is_active: true,
            transform: ImageTranforms::default(),
            image_props: ImageProps::default()
        }
    }
}


#[derive(Clone)]
pub struct ImagePreviewPad {
    pub width: f32, 
    pub height: f32,
    pub texture: OutputTexture,
}

impl Hash for ImagePreviewPad {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
        self.texture.hash(state);
    }
}

impl Default for ImagePreviewPad {
    fn default() -> Self {
        Self {
            width: 50.0,
            height: 40.0,
            texture: OutputTexture::default(),
        }
    }
}


#[derive(Clone, Copy)]
pub struct MainImageRectSetting {
    pub is_dragging: bool,
    pub image_plot_rect: egui::Rect,
    pub right_side_drag_pos: Pos2,
    pub right_side_drag_offset: f32,
    pub bottom_side_drag_pos: Pos2,
    pub bottom_side_drag_offset: f32,
}

impl Hash for MainImageRectSetting {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_dragging.hash(state);
        self.image_plot_rect.min.x.to_bits().hash(state);
        self.image_plot_rect.min.y.to_bits().hash(state);
        self.image_plot_rect.max.x.to_bits().hash(state);
        self.image_plot_rect.max.y.to_bits().hash(state);
        self.right_side_drag_pos.x.to_bits().hash(state);
        self.right_side_drag_pos.y.to_bits().hash(state);
        self.right_side_drag_offset.to_bits().hash(state);
        self.bottom_side_drag_pos.x.to_bits().hash(state);
        self.bottom_side_drag_pos.y.to_bits().hash(state);
        self.bottom_side_drag_offset.to_bits().hash(state);
    }
}

impl Default for MainImageRectSetting {
    fn default() -> Self {
        let starting_pos = Pos2::new(10., 30.);
        let size: Vec2 = Vec2::new(1000., 600.);
        Self {
            is_dragging: false,
            image_plot_rect: egui::Rect { min: starting_pos, max: starting_pos + size },
            right_side_drag_pos: Pos2::ZERO,
            bottom_side_drag_pos: Pos2::ZERO,
            right_side_drag_offset: 0.0,
            bottom_side_drag_offset: 0.0
        }
    }
}

#[derive(Clone, Copy)]
pub struct ActiveImageSettings {
    pub transforms: ImageTranforms,
    pub props: ImageProps,
    pub croped_modified: CropedImageModified,
    pub is_dragging: bool,
    pub drag_offset: Vec2
}

impl Hash for ActiveImageSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.transforms.hash(state);
        self.props.hash(state);
        self.croped_modified.hash(state);
        self.is_dragging.hash(state);
        self.drag_offset.x.to_bits().hash(state);
        self.drag_offset.y.to_bits().hash(state);
    }
}

impl Default for ActiveImageSettings {
    fn default() -> Self {
        Self {
            transforms: ImageTranforms::default(),
            props: ImageProps::default(),
            croped_modified: CropedImageModified::default(),
            is_dragging: false,
            drag_offset: Vec2::ZERO
        }
    }
    
}
#[derive(Clone, Copy, PartialEq)]
pub struct CropedImage {
   pub top: f32,
   pub left: f32, 
   pub bottom: f32,
   pub right: f32
}

impl Default for CropedImage  {
    fn default() -> Self {
        Self {
           top: 0.,
           left: 0.,
           bottom: 0.,
           right: 0.
        }
    }
}

impl Hash for CropedImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.top.to_bits().hash(state);
        self.left.to_bits().hash(state);
        self.bottom.to_bits().hash(state);
        self.right.to_bits().hash(state);
       
    }
}

impl CropedImage {
    pub fn new(top: f32, left: f32, bottom: f32, right: f32) -> Self {
        Self { top, left, bottom, right }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CropImageDragState {
    pub is_dragging: bool,
    pub drag_pos: Pos2, 
    pub drag_offset: Vec2
}

impl Default for CropImageDragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            drag_pos: Pos2::ZERO,
            drag_offset: Vec2::ZERO
        }
    }
} 

#[derive(Clone, Copy, PartialEq)]
pub struct CropedImageModified {
    pub is_modifying: bool,
    pub max_scale: f32,
    pub crop_drag_state: CropImageDragState,
    pub prev_modified: Option<CropRect>,
    pub current_modified: Option<CropRect>,
    pub dest_modified: Option<CropRect>
}

impl Default for CropedImageModified {
    fn default() -> Self {
        Self {
            is_modifying: false,
            max_scale: 1.,
            crop_drag_state: CropImageDragState::default(),
            prev_modified: None,
            current_modified: None,
            dest_modified: None
        }
    }
}

impl Hash for CropedImageModified {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_modifying.hash(state);
        self.prev_modified.hash(state);
        self.current_modified.hash(state);
    }
}
#[derive(Clone, PartialEq)]
pub struct ImageSortingModify {
    pub is_sorting: bool,
    pub is_draging: bool,
    pub drag_pos: Pos2,
    pub drag_start: Pos2,
    pub from_image: Option<usize>,
    pub image_vec: Vec<OutputTexture>
}



impl Default for ImageSortingModify {
    fn default() -> Self {
        Self {
            is_sorting: false,
            from_image: None,
            is_draging: false,
            drag_start: Pos2::ZERO,
            drag_pos: Pos2::ZERO,
            image_vec: Vec::new()
        }
    }
}

