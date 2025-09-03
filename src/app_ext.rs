use std::path::PathBuf;

use egui::{Align2, Color32, ColorImage, CursorIcon, Layout, Pos2, Sense, Stroke, TextureId, TextureOptions, Vec2, Vec2b};
use rfd::FileDialog;

use crate::resource::*;


pub trait AppExt {
    fn new(cc: &eframe::CreationContext<'_>) -> Self;
    fn import_image(&mut self,  ctx: &egui::Context);
    fn select_image(&mut self, idx: usize);
    fn add_image_to_state(&mut self);
    fn draw_image_to_board(
        &mut self, 
        ui: &mut egui::Ui, 
        image: &mut OutputTexture
    ) -> egui::Rect;
    fn add_drag_events_to_image(
        &mut self, 
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        texture_rect: egui::Rect,
        idx: usize
    );
    fn add_main_image_rect_setting_control(
        &mut self, 
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    );
    fn update_images_by_active_image(&mut self);
    fn update_croped_image_tranform(&mut self);
    fn display_image_for_cropping(&mut self, ui: &mut egui::Ui, image: &mut OutputTexture) -> egui::Rect;
    fn display_cropping_image_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, raw_rect: egui::Rect);
    fn remove_image(&mut self, image_id: TextureId);
    fn display_sorting_images(&mut self, ctx: &egui::Context);
}

#[derive(Default)]
pub struct App {
    pub main_image_settings: MainImageRectSetting,
    pub active_image: Option<ActiveImage>,
    pub output_textures_vec: Vec<OutputTexture>,
    pub image_preview_pads: Vec<ImagePreviewPad>,
    pub active_image_settings: ActiveImageSettings,
    pub image_sorting_modify: ImageSortingModify,
}

impl AppExt for App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This gives us image support:
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);
        Self {
            active_image: None,
            main_image_settings: MainImageRectSetting::default(),
            output_textures_vec: Vec::new(),
            active_image_settings: ActiveImageSettings::default(),
            image_preview_pads: Vec::new(),
            image_sorting_modify: ImageSortingModify::default(),

        }
    }
    fn import_image(&mut self, ctx: &egui::Context) {
        let file_path: Option<PathBuf> = FileDialog::new()
            .add_filter("Image", &["png", "jpeg", "jpg"])
            .pick_file();
        match file_path {
            Some(path) => {
                let reader = image::ImageReader::open(path.clone()).unwrap();
                if let Ok(image) = reader.decode()  {
                    let image_size = Vec2::new(image.width() as f32, image.height() as f32);
                    let image_ratio = calc_ratio(image_size.clone());
                    let original_scale = calc_orignal_scale(Vec2::new(image.width() as f32, image.height() as f32), self.main_image_settings.image_plot_rect.size());
                    let color_image = ColorImage::from_rgba_unmultiplied(
                        [image.width() as _, image.height() as _],
                        image.to_rgba8().as_flat_samples().as_slice(),
                    );
                    let texture_handled =  ctx.load_texture(
                        "imported_image",
                        color_image.clone(),
                        TextureOptions::default(),
                    );
                    let loaded_texture = Some(OutputTexture {
                        image: Some(image.clone()),
                        file_path: Some(path),
                        original_scale: original_scale.clone(),
                        texture_handle: Some(texture_handled),
                        image_ratio: image_ratio,
                        transform: ImageTranforms { 
                            size: image_size,
                            original_scale: original_scale,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    // self.active_image_settings.transforms.pos = Pos2 { x: 0.0, y: 0.0 };
                    // self.active_image_settings.transforms.scale = 1.;
                    self.active_image_settings = ActiveImageSettings::default();
                    self.active_image_settings.transforms = loaded_texture.clone().unwrap().transform.clone();
                    self.active_image_settings.drag_offset = Vec2::new(0.0, 0.0);
                    self.active_image =  Some(ActiveImage::new(loaded_texture.clone().unwrap().texture_handle.as_ref().unwrap().id() ));
                    self.output_textures_vec.push(loaded_texture.clone().unwrap());
                    self.image_preview_pads.push(ImagePreviewPad {
                        
                        texture: loaded_texture.clone().unwrap_or_default(),
                    }); 
                };
            },
            _ => {
                return;
            }
        } 
        
        // self.load_color_image();
        // self.load_texture(&egui::Context::default());
        
        // self.current_output_texture.as_ref().and_then(|ot| ot.image.clone())
    }

    fn select_image(&mut self, idx: usize) {
        // let picked_pad = self.image_preview_pads[idx].clone();
        if self.active_image_settings.croped_modified.is_modifying {
            return;
        }
        let pick_texture: &OutputTexture = &self.output_textures_vec[idx];
        self.active_image_settings.transforms = pick_texture.transform.clone();
        self.active_image_settings.props = pick_texture.image_props.clone();
        self.active_image_settings.croped_modified = CropedImageModified::default();
        self.active_image = Some(ActiveImage::new(pick_texture.texture_handle.as_ref().unwrap().id()));
        self.active_image_settings.drag_offset = Vec2::new(0.0, 0.0);
        // Implement selection logic if needed
    }

    fn add_image_to_state(&mut self) {
        // Implement logic to add image to state if needed
    }
    fn draw_image_to_board(
        &mut self, 
        ui: &mut egui::Ui,
        image: &mut OutputTexture
    ) -> egui::Rect {
        let main_image_rect = self.main_image_settings.image_plot_rect;
        let texture = image.texture_handle.as_ref().unwrap();
        let croped = image.transform.croped.unwrap_or_default();
        // let texture_props = &image.image_props;
        // Calculate the scaled size while maintaining aspect ratio
        let img_size = texture.size_vec2();
        
        // Scale the image based on the board size and scale factor

        // let scaled_height = main_image_rect.height() * image.scale;
        // let scaled_width = scaled_height * aspect_ratio;
        let scaled_height = img_size.y * image.original_scale * image.transform.scale;
        let scaled_width = img_size.x * image.original_scale * image.transform.scale;
        
        // Center the image in the board
        let center_x = main_image_rect.min.x + main_image_rect.width() / 2.0;
        let center_y = main_image_rect.min.y + main_image_rect.height() / 2.0;
        
        // Apply pan offset
        let offset = image.transform.pos;
        
        // Calculate the unclamped texture rectangle with panning
        let unclamped_rect = egui::Rect::from_center_size(
            Pos2::new(
                center_x + offset.x,
                center_y + offset.y
            ),
            Vec2::new(scaled_width, scaled_height)
        );
       
        // Clamp the texture rectangle to the board boundaries
        let texture_rect = egui::Rect {
            min: Pos2::new(
                (unclamped_rect.min.x + croped.left * image.transform.scale).max(main_image_rect.min.x).min(main_image_rect.max.x),
                (unclamped_rect.min.y + croped.top * image.transform.scale).max(main_image_rect.min.y ).min(main_image_rect.max.y)
            ),
            max: Pos2::new(
                 (unclamped_rect.max.x + croped.right * image.transform.scale).min(main_image_rect.max.x).max(main_image_rect.min.x),
                 (unclamped_rect.max.y+ croped.bottom * image.transform.scale).min(main_image_rect.max.y).max(main_image_rect.min.y)
            ),
        };
        // Calculate UV coordinates based on the clamped rectangle
        let uv_rect = egui::Rect::from_min_max(
            egui::Pos2::new(
                ((texture_rect.min.x - (unclamped_rect.min.x)) / scaled_width).max(0.0),
                ((texture_rect.min.y - (unclamped_rect.min.y)) / scaled_height).max(0.0)
            ),
            egui::Pos2::new(
                ((texture_rect.max.x - (unclamped_rect.min.x)) / scaled_width).min(1.0),
                ((texture_rect.max.y - (unclamped_rect.min.y)) / scaled_height).min(1.0)
            )
        );
        
        image.image_props.uv_rect = uv_rect;
        // Calculate rotation
        ui.painter().image(
            texture.id(), 
            texture_rect, 
            uv_rect, 
            Color32::from_white_alpha((image.transform.opacity * 255.0) as u8)
        );
        
        // Create triangles
        texture_rect
    }
    fn add_drag_events_to_image(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, texture_rect:egui::Rect, idx: usize ) {

        let click_drag_respone = ui.allocate_rect(texture_rect, Sense::click_and_drag());
        if click_drag_respone.hovered() {
            ctx.set_cursor_icon(CursorIcon::Grab);
        }
        if click_drag_respone.clicked() {
          
            self.select_image(idx);
        
        }
        if click_drag_respone.drag_started() {
            self.select_image(idx);

            self.active_image_settings.is_dragging = true;
            
            self.active_image_settings.drag_offset = click_drag_respone.interact_pointer_pos().unwrap() as Pos2 - self.active_image_settings.transforms.pos as Pos2;
        }
        if self.active_image_settings.is_dragging && click_drag_respone.dragged() {
            ctx.set_cursor_icon(CursorIcon::Grabbing);
            if let Some(pointer_pos) = click_drag_respone.interact_pointer_pos() {
                self.active_image_settings.transforms.pos = pointer_pos - self.active_image_settings.drag_offset;
                // let texture = self.find_texture(idx)
                if self.active_image.is_some() {
                    for output_texture in self.output_textures_vec.iter_mut() {
                        if self.active_image.as_ref().unwrap().get_texture_id() == output_texture.texture_handle.as_ref().unwrap().id() {
                            output_texture.transform.pos = self.active_image_settings.transforms.pos.clone();
                            
                        }
                    }   
                }
            }
        }
        if click_drag_respone.drag_stopped() {
            ctx.set_cursor_icon(CursorIcon::Alias);
            self.active_image_settings.is_dragging = false;
        }
    }

    fn add_main_image_rect_setting_control(
        &mut self, 
        ctx: &egui::Context,
        ui: &mut egui::Ui,
    ) {
        // Right side line
        ui.painter().line(Vec::from([Pos2::new(self.main_image_settings.image_plot_rect.max.x + 5., self.main_image_settings.image_plot_rect.min.y), Pos2::new(self.main_image_settings.image_plot_rect.max.x + 5., self.main_image_settings.image_plot_rect.max.y + 10.)]), Stroke::new(5., Color32::WHITE));
        // Bottom side line
        ui.painter().line(Vec::from([Pos2::new(self.main_image_settings.image_plot_rect.min.x, self.main_image_settings.image_plot_rect.max.y + 5.), Pos2::new(self.main_image_settings.image_plot_rect.max.x + 10., self.main_image_settings.image_plot_rect.max.y + 5.)]), Stroke::new(5., Color32::WHITE));
        
        let right_side_drag_response = ui.allocate_rect(
            egui::Rect { 
                min: Pos2 { x: self.main_image_settings.image_plot_rect.max.x - 2.5, y: 0.0 },
                max: Pos2 { x: self.main_image_settings.image_plot_rect.max.x + 2.5, y: self.main_image_settings.image_plot_rect.max.y }
            }, 
            Sense::click_and_drag()
        );
        let bottom_side_drag_response = ui.allocate_rect(
            egui::Rect { 
                min: Pos2 { x: 0., y: self.main_image_settings.image_plot_rect.max.y - 2.5 },
                max: Pos2 { x: self.main_image_settings.image_plot_rect.max.x, y: self.main_image_settings.image_plot_rect.max.y + 2.5}
            }, 
            Sense::click_and_drag()
        );
    
        if right_side_drag_response.hovered() {
            ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);

        }
        if bottom_side_drag_response.hovered() {
            ctx.set_cursor_icon(CursorIcon::ResizeVertical);
        } 

        if right_side_drag_response.drag_started() {
            let interact_point = right_side_drag_response.interact_pointer_pos().unwrap() as Pos2;
            self.main_image_settings.right_side_drag_pos = self.main_image_settings.image_plot_rect.max;
            self.main_image_settings.is_dragging = true;
            self.main_image_settings.right_side_drag_offset = interact_point.x - (self.main_image_settings.right_side_drag_pos as Pos2).x;
        }
        
        if self.main_image_settings.is_dragging && right_side_drag_response.dragged() {
            ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
            if let Some(pointer_pos) = right_side_drag_response.interact_pointer_pos() {
                self.main_image_settings.right_side_drag_pos = pointer_pos - Vec2::new(self.main_image_settings.right_side_drag_offset, 0.0);
                self.main_image_settings.image_plot_rect.max.x = self.main_image_settings.right_side_drag_pos.x;
                // let texture = self.find_texture(idx)
                
            }
        }
        if right_side_drag_response.drag_stopped() {
            ctx.set_cursor_icon(CursorIcon::Alias);
            self.main_image_settings.is_dragging = false;
        }

            if bottom_side_drag_response.drag_started() {
            let interact_point = bottom_side_drag_response.interact_pointer_pos().unwrap() as Pos2;
            self.main_image_settings.bottom_side_drag_pos = self.main_image_settings.image_plot_rect.max;
            self.main_image_settings.is_dragging = true;
            self.main_image_settings.bottom_side_drag_offset = interact_point.y - (self.main_image_settings.bottom_side_drag_pos as Pos2).y ;
        }
        
        if self.main_image_settings.is_dragging && bottom_side_drag_response.dragged() {
            ctx.set_cursor_icon(CursorIcon::ResizeVertical);
            if let Some(pointer_pos) = bottom_side_drag_response.interact_pointer_pos() {
                self.main_image_settings.bottom_side_drag_pos = pointer_pos - Vec2::new(0.0, self.main_image_settings.bottom_side_drag_offset);
                self.main_image_settings.image_plot_rect.max.y = self.main_image_settings.bottom_side_drag_pos.y;
                // let texture = self.find_texture(idx)
                
            }
        }
        if bottom_side_drag_response.drag_stopped() {
            ctx.set_cursor_icon(CursorIcon::Alias);
            self.main_image_settings.is_dragging = false;
        }
    }
    fn update_images_by_active_image(&mut self) {

       
       let find_image = self.output_textures_vec.iter_mut().find(|image| image.texture_handle.as_ref().unwrap().id() == self.active_image.as_ref().unwrap().get_texture_id());
       if let Some(image) = find_image {
            image.transform.scale = self.active_image_settings.transforms.scale.clone();
            image.transform.opacity = self.active_image_settings.transforms.opacity.clone();
            image.transform.rotation = self.active_image_settings.transforms.rotation.clone();
            image.transform.pos = self.active_image_settings.transforms.pos.clone();
            self.active_image_settings.props.to_owned().uv_rect = image.image_props.uv_rect.clone();
       }
    }
    fn display_cropping_image_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, raw_rect: egui::Rect) {
        let mut current_modified = self.active_image_settings.croped_modified.current_modified.unwrap_or_default().clone();
        let main_rect = self.main_image_settings.image_plot_rect;
        let crop_line_stroke = Stroke::new(2., Color32::from_rgb(255, 255, 255));

        // raw rect
        let mut clone_raw_rect = raw_rect.clone();
        let unclamped_raw_rect = raw_rect.clone();
        clone_raw_rect.min = clone_raw_rect.min.clamp(self.main_image_settings.image_plot_rect.min, self.main_image_settings.image_plot_rect.max);
        clone_raw_rect.max = clone_raw_rect.max.clamp(self.main_image_settings.image_plot_rect.min, self.main_image_settings.image_plot_rect.max);
        // ui.painter().rect_stroke(
        //     clone_raw_rect, 
        //     0.0,
        //     Stroke::new(1.0, Color32::from_rgb(255,25, 25)), 
            
        //     StrokeKind::Middle
        // );

        
        let modified_top = ((unclamped_raw_rect.min.y  + current_modified.top  ).max(main_rect.min.y)).min(main_rect.max.y);
        let modified_left = ((unclamped_raw_rect.min.x  + current_modified.left  ).max(main_rect.min.x)).min(main_rect.max.x);
        let modified_bottom = ((unclamped_raw_rect.max.y  + current_modified.bottom   ).min(main_rect.max.y)).max(main_rect.min.y);
        let modified_right = ((unclamped_raw_rect.max.x  + current_modified.right  ).min(main_rect.max.x)).max(main_rect.min.x);
        
        
            
        if !(raw_rect.min.x < main_rect.min.x || raw_rect.min.y < main_rect.min.y) {
            let top_left_rect = egui::Rect {
                min: Pos2::new(modified_left , modified_top ),
                max: Pos2::new(modified_left + 15.0, modified_top + 15.0).min(Pos2::new(modified_right, modified_bottom ))
            };
            ui.painter().rect_filled(top_left_rect, 0.0, Color32::from_rgb(255, 255, 255));
            let top_drag_sense = ui.allocate_rect(
                top_left_rect.clone(), Sense::click_and_drag()
            );


            if top_drag_sense.hovered() {
                ctx.set_cursor_icon(CursorIcon::Grabbing);
            };

            if top_drag_sense.drag_started() {
                self.active_image_settings.croped_modified.crop_drag_state.is_dragging = true;
                let starting_pos =  unclamped_raw_rect.min + Vec2::new(current_modified.left, current_modified.top);
                // let starting_pos =  unclamped_raw_rect.min + Vec2::new(self.active_image_settings.croped_modified.current_modified.unwrap_or_default().left, self.active_image_settings.croped_modified.current_modified.unwrap_or_default().top);
                let interaction_pos = top_drag_sense.interact_pointer_pos().unwrap() as Pos2;
                // let starting_pos =  unclamped_raw_rect.min.y + self.active_image_settings.croped_modified.current_modified.unwrap_or_default().top;
                self.active_image_settings.croped_modified.crop_drag_state.drag_pos = starting_pos;
                self.active_image_settings.croped_modified.crop_drag_state.drag_offset = interaction_pos - self.active_image_settings.croped_modified.crop_drag_state.drag_pos;
            }
            if self.active_image_settings.croped_modified.crop_drag_state.is_dragging && top_drag_sense.dragged() {
                ctx.set_cursor_icon(CursorIcon::Grabbing);
                if let Some(pointer_pos) = top_drag_sense.interact_pointer_pos() {
                    let clamp_pos = pointer_pos.clamp(
                        unclamped_raw_rect.min ,
                        unclamped_raw_rect.max + Vec2::new(current_modified.right, current_modified.bottom)
                    );
                    let crop = (clamp_pos - raw_rect.min.to_vec2()) - (self.active_image_settings.croped_modified.crop_drag_state.drag_offset ); 
                    // pos = pos ;
                    // let top =   ;
                    // pos = pos.clamp(raw_rect.min, raw_rect.max);
                    current_modified.top = crop.y.min(raw_rect.height() - current_modified.bottom).max(0.);
                    current_modified.left = crop.x.min(raw_rect.width() - current_modified.right).max(0.);
                    self.active_image_settings.croped_modified.current_modified = Some(current_modified.clone());
                 
                }
            }
            if top_drag_sense.drag_stopped() {
                ctx.set_cursor_icon(CursorIcon::Alias);
                self.active_image_settings.croped_modified.crop_drag_state.is_dragging = false;
            }


        }

        if !(raw_rect.max.x > main_rect.max.x || raw_rect.max.y > main_rect.max.y) {
            let bottom_right_rect = egui::Rect {
                min: Pos2::new(modified_right - 15.0, modified_bottom - 15.0 ).max(Pos2::new(modified_left , modified_top)),
                max: Pos2::new(modified_right, modified_bottom )
            };
            ui.painter().rect_filled(bottom_right_rect.clone(), 0.0, Color32::from_rgb(255, 255, 255));
            let bottom_drag_sense = ui.allocate_rect(
                bottom_right_rect.clone(), Sense::click_and_drag()
            );


            if bottom_drag_sense.hovered() {
                ctx.set_cursor_icon(CursorIcon::Grabbing);
            };

            if bottom_drag_sense.drag_started() {
                self.active_image_settings.croped_modified.crop_drag_state.is_dragging = true;
                let starting_pos =  unclamped_raw_rect.max + Vec2::new(current_modified.right, current_modified.bottom);
                // let starting_pos =  unclamped_raw_rect.min + Vec2::new(self.active_image_settings.croped_modified.current_modified.unwrap_or_default().left, self.active_image_settings.croped_modified.current_modified.unwrap_or_default().top);
                let interaction_pos = bottom_drag_sense.interact_pointer_pos().unwrap() as Pos2;
                // let starting_pos =  unclamped_raw_rect.min.y + self.active_image_settings.croped_modified.current_modified.unwrap_or_default().top;
                self.active_image_settings.croped_modified.crop_drag_state.drag_pos = starting_pos;
                self.active_image_settings.croped_modified.crop_drag_state.drag_offset = interaction_pos - self.active_image_settings.croped_modified.crop_drag_state.drag_pos;
            }
            if self.active_image_settings.croped_modified.crop_drag_state.is_dragging && bottom_drag_sense.dragged() {
                ctx.set_cursor_icon(CursorIcon::Grabbing);
                if let Some(pointer_pos) = bottom_drag_sense.interact_pointer_pos() {
                    let clamp_pos = pointer_pos.clamp(
                        unclamped_raw_rect.min + Vec2::new(current_modified.left, current_modified.top),
                        unclamped_raw_rect.max
                    );
                    let crop = (clamp_pos - raw_rect.max.to_vec2()) - (self.active_image_settings.croped_modified.crop_drag_state.drag_offset ); 
                    // pos = pos ;
                    // let top =   ;
                    // pos = pos.clamp(raw_rect.min, raw_rect.max);
                    current_modified.bottom = crop.y.max(-(raw_rect.height() - current_modified.top)).min(0.);
                    current_modified.right = crop.x.max(-(raw_rect.width() - current_modified.left)).min(0.);
                    self.active_image_settings.croped_modified.current_modified = Some(current_modified.clone());
                 
                }
            }
            if bottom_drag_sense.drag_stopped() {
                ctx.set_cursor_icon(CursorIcon::Alias);
                self.active_image_settings.croped_modified.crop_drag_state.is_dragging = false;
            }
        }
        // }
        if !(raw_rect.min.y  < main_rect.min.y)  { 
            ui.painter().line(Vec::from([
                Pos2::new(clone_raw_rect.min.x, clone_raw_rect.min.y ),
                Pos2::new(clone_raw_rect.max.x, clone_raw_rect.min.y )
            ]), Stroke::new(1.0, Color32::from_rgb(255,25, 25)));

        }
            // raw line
            
        let _top_line = ui.painter().line(Vec::from([
            Pos2::new(modified_left,  modified_top),
            Pos2::new(modified_right, modified_top)
        ]), crop_line_stroke);
        
            
        
        
        if !(raw_rect.min.x < main_rect.min.x) {
            // raw line
            ui.painter().line(Vec::from([
                Pos2::new(clone_raw_rect.min.x, clone_raw_rect.min.y ),
                Pos2::new(clone_raw_rect.min.x, clone_raw_rect.max.y )
            ]), Stroke::new(1.0, Color32::from_rgb(255,25, 25)));
        }
            // let left_line = ui.painter().line(Vec::from([
            //     Pos2::new(clone_raw_rect.min.x + current_modified.left, clone_raw_rect.min.y + current_modified.top),
            //     Pos2::new(clone_raw_rect.min.x + current_modified.left, clone_raw_rect.max.y + current_modified.bottom)
            // ]), crop_line_stroke);
            let _left_line = ui.painter().line(Vec::from([
                Pos2::new(modified_left, modified_top),
                Pos2::new(modified_left, modified_bottom)
            ]), crop_line_stroke);
        // }
        
        if !(raw_rect.max.y > main_rect.max.y) {
            // raw line
            ui.painter().line(Vec::from([
                Pos2::new(clone_raw_rect.min.x, clone_raw_rect.max.y ),
                Pos2::new(clone_raw_rect.max.x, clone_raw_rect.max.y )
            ]), Stroke::new(1.0, Color32::from_rgb(255,25, 25)));
        }
            // let bottom_line = ui.painter().line(Vec::from([
            //     Pos2::new(clone_raw_rect.min.x + current_modified.left, clone_raw_rect.max.y + current_modified.bottom ),
            //     Pos2::new(clone_raw_rect.max.x + current_modified.right, clone_raw_rect.max.y + current_modified.bottom)
            // ]), crop_line_stroke);
            let _bottom_line = ui.painter().line(Vec::from([
                Pos2::new(modified_left, modified_bottom ),
                Pos2::new(modified_right, modified_bottom)
            ]), crop_line_stroke);
        // }
    
        if !(raw_rect.max.x > main_rect.max.x) {
             // raw line
            ui.painter().line(Vec::from([
                Pos2::new(clone_raw_rect.max.x, clone_raw_rect.min.y ),
                Pos2::new(clone_raw_rect.max.x, clone_raw_rect.max.y )
            ]), Stroke::new(1.0, Color32::from_rgb(255,25, 25)));
        }
            // let right_line = ui.painter().line(Vec::from([
            //     Pos2::new(clone_raw_rect.max.x + current_modified.right, clone_raw_rect.min.y + current_modified.top),
            //     Pos2::new(clone_raw_rect.max.x + current_modified.right, clone_raw_rect.max.y + current_modified.bottom)
            // ]), crop_line_stroke);
        let _right_line = ui.painter().line(Vec::from([
            Pos2::new(modified_right, modified_top),
            Pos2::new(modified_right, modified_bottom)
        ]), crop_line_stroke);
        // let texture_rect = self.active_image_settings;
        // ui.painter().line(points, stroke)
    }
    fn update_croped_image_tranform(&mut self) {
        let current_crop = self.active_image_settings.croped_modified.current_modified.clone();
        // let scaled = self.active_image_settings.transforms.scale.clone();
        let scaled = self.active_image_settings.croped_modified.max_scale.clone();
        let multiply = 1. / scaled;
        if let Some(croped) = current_crop {
            let dest = CropRect{top: croped.top * multiply, left: croped.left * multiply , bottom: croped.bottom * multiply, right: croped.right * multiply };
            self.active_image_settings.croped_modified.dest_modified = Some(dest);
        } else {
            self.active_image_settings.croped_modified.dest_modified = None;
        };
        self.active_image_settings.transforms.croped = self.active_image_settings.croped_modified.dest_modified.clone();
        for output_texture in self.output_textures_vec.iter_mut() {
            if self.active_image.as_ref().unwrap().get_texture_id() == output_texture.texture_handle.as_ref().unwrap().id() {
                output_texture.transform.croped = self.active_image_settings.transforms.croped.clone();
            }
        }    
    }
    fn display_image_for_cropping(&mut self, ui: &mut egui::Ui, image: &mut OutputTexture) -> egui::Rect {
        let main_image_rect = self.main_image_settings.image_plot_rect;
        let max_scale = self.active_image_settings.croped_modified.max_scale.clone() * image.transform.original_scale;
        // let min_main_image_size = main_image_rect.width().min(main_image_rect.height());
        let texture = image.texture_handle.as_ref().unwrap();
        // let max_image_size = texture.size()[0].max(texture.size()[1]);
        // let croped = image.transform.croped.unwrap_or_default();
        let croped = self.active_image_settings.croped_modified.current_modified.unwrap_or_default();
        // let texture_props = &image.image_props;
        // Calculate the scaled size while maintaining aspect ratio
        let img_size = texture.size_vec2();
        
        // Scale the image based on the board size and scale factor

        // let scaled_height = main_image_rect.height() * image.scale;
        // let scaled_width = scaled_height * aspect_ratio;
        let scaled_height = img_size.y  * max_scale;
        let scaled_width = img_size.x  * max_scale;
        
        // Center the image in the board
        let center_x = main_image_rect.min.x + main_image_rect.width() / 2.0;
        let center_y = main_image_rect.min.y + main_image_rect.height() / 2.0;
        
        // Apply pan offset
        
        // Calculate the unclamped texture rectangle with panning
        let unclamped_rect = egui::Rect::from_center_size(
            Pos2::new(
                center_x ,
                center_y
            ),
            Vec2::new(scaled_width, scaled_height)
        );
       let background_texture_rect = egui::Rect {
            min: Pos2::new(
                (unclamped_rect.min.x ).max(main_image_rect.min.x).min(main_image_rect.max.x),
                (unclamped_rect.min.y ).max(main_image_rect.min.y ).min(main_image_rect.max.y)
            ),
            max: Pos2::new(
                 (unclamped_rect.max.x).min(main_image_rect.max.x).max(main_image_rect.min.x),
                 (unclamped_rect.max.y).min(main_image_rect.max.y).max(main_image_rect.min.y)
            ),
        };
        // Calculate UV coordinates based on the clamped rectangle
        let uv_rect = egui::Rect::from_min_max(
            egui::Pos2::new(
                ((background_texture_rect.min.x - (unclamped_rect.min.x)) / scaled_width).max(0.0),
                ((background_texture_rect.min.y - (unclamped_rect.min.y)) / scaled_height).max(0.0)
            ),
            egui::Pos2::new(
                ((background_texture_rect.max.x - (unclamped_rect.min.x)) / scaled_width).min(1.0),
                ((background_texture_rect.max.y - (unclamped_rect.min.y)) / scaled_height).min(1.0)
            )
        );
        
        // image.image_props.uv_rect = uv_rect;
        // Calculate rotation
        ui.painter().image(
            texture.id(), 
            background_texture_rect, 
            uv_rect, 
            Color32::from_white_alpha((0.5 * 255.0) as u8)
        );
        
        // Clamp the texture rectangle to the board boundaries
        let texture_rect = egui::Rect {
            min: Pos2::new(
                (unclamped_rect.min.x + croped.left).max(main_image_rect.min.x).min(main_image_rect.max.x),
                (unclamped_rect.min.y + croped.top).max(main_image_rect.min.y ).min(main_image_rect.max.y)
            ),
            max: Pos2::new(
                 (unclamped_rect.max.x + croped.right).min(main_image_rect.max.x).max(main_image_rect.min.x),
                 (unclamped_rect.max.y+ croped.bottom).min(main_image_rect.max.y).max(main_image_rect.min.y)
            ),
        };
        // Calculate UV coordinates based on the clamped rectangle
        let uv_rect = egui::Rect::from_min_max(
            egui::Pos2::new(
                ((texture_rect.min.x - (unclamped_rect.min.x)) / scaled_width).max(0.0),
                ((texture_rect.min.y - (unclamped_rect.min.y)) / scaled_height).max(0.0)
            ),
            egui::Pos2::new(
                ((texture_rect.max.x - (unclamped_rect.min.x)) / scaled_width).min(1.0),
                ((texture_rect.max.y - (unclamped_rect.min.y)) / scaled_height).min(1.0)
            )
        );
        
        image.image_props.uv_rect = uv_rect;
        // Calculate rotation
        ui.painter().image(
            texture.id(), 
            texture_rect, 
            uv_rect, 
            Color32::from_white_alpha((image.transform.opacity * 255.0) as u8)
        );
        
        unclamped_rect
    }
    fn remove_image(&mut self, image_id: TextureId) {
        let find_image = self.output_textures_vec.iter_mut().enumerate().find(|(_index, image)| image.texture_handle.as_ref().unwrap().id() == image_id);
        if let Some((idx, _image)) = find_image {
            self.output_textures_vec.remove(idx);
            // image.texture_handle.as_ref().unwrap();
            if let Some(check_active) = self.active_image.as_ref() {
                if check_active.get_texture_id() == image_id {
                    self.active_image = None;
                }
            }
        }
    }
    fn display_sorting_images(&mut self, ctx: &egui::Context ) {
        let image_vec = &mut self.output_textures_vec;
        egui::Window::new("Sorting images")
            .anchor(Align2::RIGHT_CENTER, Vec2::ZERO)
            .title_bar(false)
            
            // .min_height(400.0)
            // .default_height(400.0)
            .resizable(Vec2b::new(false, false))
            .show(ctx, |ui| {
                let margin = 5.;
                let image_rect_size = Vec2::new(250.0, 60.0);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(egui::Align::Max), |ui| {
                        ui.label(format!("Your images list:  {}", image_vec.len()));
                        let finish_button = ui.button("Done");
                        if  finish_button.clicked() {
                            self.image_sorting_modify.is_sorting = false
                        };
                    });
                });
                ui.separator();
                let (container_rect, _) = ui.allocate_exact_size(Vec2::new(350., (image_rect_size.y + margin * 2.) * image_vec.len() as f32), Sense::click());
                
                for (idx, image_preview) in image_vec.clone().iter_mut().enumerate() { 
                   
                    let mut image_rect = egui::Rect::from_min_max(
                        Pos2::new(container_rect.min.x + margin, container_rect.min.y + margin + (image_rect_size.y + margin * 2.) * idx  as f32), 
                        Pos2::new(container_rect.min.x + margin + image_rect_size.x, container_rect.min.y + margin + (image_rect_size.y + margin * 2.) * idx  as f32 + image_rect_size.y)
                    );

                    let scaled_size = Vec2::new(image_rect_size.x, image_rect_size.x / image_preview.image_ratio);
                    
                    let mut uv_rect = egui::Rect::from_min_max(
                        Pos2::new(0.0, 0.5 - (image_rect_size.y / scaled_size.y) / 2.),
                        Pos2::new(1.0, 0.5 + (image_rect_size.y / scaled_size.y) / 2.)
                    );
                    
                    if self.image_sorting_modify.is_sorting && self.image_sorting_modify.is_draging {
                        let new_idx = ((self.image_sorting_modify.drag_pos.y - container_rect.min.y ) / (image_rect_size.y + margin * 2.)).round() as usize;
                        if new_idx > 0. as usize && idx == (new_idx.clone() - 1) {
                            image_rect.max.y -= 5.;
                            uv_rect.max.y -= 5. / scaled_size.y;
                        }

                        if new_idx < (image_vec.len().clone() ) && idx == new_idx.clone()  {
                            image_rect.min.y += 5.;
                            uv_rect.min.y += 5. / scaled_size.y;

                        }
                    }
                    // let new_rect_height = image_rect.height();
                    ui.painter().image(
                        image_preview.texture_handle.as_ref().unwrap().id(),
                        image_rect,
                        uv_rect,
                        Color32::from_white_alpha(255)
                    );


                }

                for (idx, image_preview) in image_vec.clone().iter_mut().enumerate() { 
                   
                   
                    let image_rect = egui::Rect::from_min_max(
                        Pos2::new(container_rect.min.x + margin, container_rect.min.y + margin + (image_rect_size.y + margin * 2.) * idx  as f32), 
                        Pos2::new(container_rect.min.x + margin + image_rect_size.x, container_rect.min.y + margin + (image_rect_size.y + margin * 2.) * idx  as f32 + image_rect_size.y)
                    );
                    let image_drag_sense = ui.allocate_rect(image_rect, Sense::click_and_drag());
                    if image_drag_sense.clicked() {
                        self.image_sorting_modify.from_image = Some(idx);
                       
                    }
                    if image_drag_sense.hovered() {
                        ctx.set_cursor_icon(CursorIcon::PointingHand);
                    }
                    if image_drag_sense.drag_started() {
                        ctx.set_cursor_icon(CursorIcon::PointingHand);

                        self.image_sorting_modify.from_image = Some(idx);
                        let starting_pos = Pos2::new(container_rect.min.x + margin * 2. + image_rect_size.x, container_rect.min.y + (image_rect_size.y + margin * 2.) * idx as f32 + image_rect_size.y / 2.);
                        self.image_sorting_modify.is_draging = true;
                        self.image_sorting_modify.drag_start = starting_pos;
 
                    }
                    if let Some(active_idx) = self.image_sorting_modify.from_image && active_idx == idx {
                        
                        let points = vec![
                            Pos2::new(container_rect.min.x + margin * 2. + image_rect_size.x, container_rect.min.y + margin + (image_rect_size.y + margin * 2.) * active_idx as f32 ),
                            Pos2::new(container_rect.min.x + margin * 2. + image_rect_size.x, container_rect.min.y + margin + (image_rect_size.y + margin * 2.) * active_idx as f32 + image_rect_size.y),
                        ];
                        ui.painter().line(points, Stroke::new(5., Color32::WHITE));
                        
                        if self.image_sorting_modify.is_draging && image_drag_sense.dragged() {
                            ctx.set_cursor_icon(CursorIcon::PointingHand);
                            let interact_pos = image_drag_sense.interact_pointer_pos().unwrap() as Pos2;
                           
                            let mut new_pos = interact_pos.clone().clamp(container_rect.min, container_rect.max);
                            let new_idx = ((new_pos.y - container_rect.min.y ) / (image_rect_size.y + margin * 2.)).round() as usize;
                            new_pos.y = new_idx as f32 * (image_rect_size.y + margin * 2.) + container_rect.min.y; 
                            new_pos.x = new_pos.x.clamp(container_rect.min.x + margin, container_rect.min.x + margin * 2. + image_rect_size.x + 25.);
                           
                            self.image_sorting_modify.drag_pos = new_pos.clone();
                            ui.painter().line(
                                vec![
                                    Pos2::new(container_rect.min.x + margin * 2. + image_rect_size.x, container_rect.min.y + (image_rect_size.y + margin * 2.) * active_idx as f32 + image_rect_size.y / 2.),
                                    Pos2::new(container_rect.min.x + margin * 2. + image_rect_size.x + 25., container_rect.min.y + (image_rect_size.y + margin * 2.) * active_idx as f32 + image_rect_size.y / 2.),
                                    Pos2::new(container_rect.min.x + margin * 2. + image_rect_size.x + 25., new_pos.y),
                                    
                                    new_pos.clone()
                                ], 
                                Stroke::new(2., Color32::WHITE)
                            );
                            ui.painter().line(
                                vec![
                                    Pos2::new(new_pos.x + 5., new_pos.y - 5.),
                                    new_pos,
                                    Pos2::new(new_pos.x + 5., new_pos.y + 5.),
                                    Pos2::new(new_pos.x + 5., new_pos.y - 5.),

                                ],
                                Stroke::new(2., Color32::WHITE)
                            );
                        }
    
                        if image_drag_sense.drag_stopped() {
                            ctx.set_cursor_icon(CursorIcon::Alias);
                            self.image_sorting_modify.is_draging = false;
                            let new_idx = ((self.image_sorting_modify.drag_pos.y - container_rect.min.y ) / (image_rect_size.y + margin * 2.)).round() as usize;
                            if let Some(current_idx) = self.image_sorting_modify.from_image {
                                if !(new_idx == current_idx) {
                                    if new_idx < current_idx {
                                        let clone_image = image_preview.clone();
                                        image_vec.remove(current_idx);
                                        image_vec.insert(new_idx, clone_image);
                                    } else {
                                        let clone_image = image_preview.clone();
                                        image_vec.remove(current_idx);
                                        image_vec.insert(new_idx - 1, clone_image);
                                    }
                                }
                            }
                        } 
                    }
                }
                
            });
    }
}



fn calc_ratio(image_size: Vec2) -> f32 {
    
    
    image_size.x / image_size.y
}

fn calc_orignal_scale(image_size: Vec2, plot_size: Vec2) -> f32 {
    let default_size = 600.;
    let image_ratio = calc_ratio(image_size);
    let plot_ratio = calc_ratio(plot_size);
    if image_ratio > plot_ratio {
        default_size / image_size.x
    } else {
        default_size / image_size.y
    }
}

pub fn calc_max_scale(image_size: Vec2, plot_size: Vec2) -> f32 {
    let default_size = plot_size.x.min(plot_size.y) - 100.;
    let image_ratio = calc_ratio(image_size);
    let plot_ratio = calc_ratio(plot_size);
    if image_ratio > plot_ratio {
        default_size / image_size.x
    } else {
        default_size / image_size.y
    }
}