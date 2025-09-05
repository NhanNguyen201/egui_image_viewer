use eframe::egui;
use egui::{CentralPanel, Color32, CursorIcon, Pos2, RichText, ScrollArea, Sense, Stroke, Vec2};
pub mod resource;
use resource::*;
pub mod app_ext;
use app_ext::*;
fn main() -> eframe::Result {
     let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 900.0])
        .with_resizable(true),
        // .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native("My Image Viewer App", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}


impl eframe::App for App {
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Import Image").clicked(){
                // let file_path = pick_file();
                self.import_image(ctx);
            }
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let main_image_settings = &mut self.main_image_settings;
                    let main_image_rect = main_image_settings.image_plot_rect;
                   
                    
                    ui.painter().rect_filled(main_image_rect, 0.0, Color32::from_rgb(200, 200, 200));
                    self.add_main_image_rect_setting_control(ctx, ui);
                    
                    if self.active_image_settings.croped_modified.is_modifying {
                        ui.painter().rect_filled(main_image_rect, 0.0, Color32::from_rgb(50, 50, 50));
                        if let Some( image) = self.output_textures_vec.clone().iter_mut().find(| image| {
                            image.texture_handle.as_ref().unwrap().id() == self.active_image.as_ref().unwrap().get_texture_id()
                        }) {
                            let  raw_rect = self.display_image_for_cropping(ui,  image);
                            // self.add_drag_events_to_image(ctx, ui, texture_rect, idx);
                            self.display_cropping_image_ui(ctx, ui, raw_rect);
                        }
                    } else {
                       
                        for (idx, output_texture) in self.output_textures_vec.clone().iter_mut().rev().enumerate() {
                            let texture_rect = self.draw_image_to_board(ui,  output_texture);
                            self.add_drag_events_to_image(ctx, ui, texture_rect, self.output_textures_vec.len() -1 -idx);
                            
                        }
                    }


                    
                    

                    
                    if self.active_image.is_some() {
                        self.update_images_by_active_image();
                    }
                });
                
                ui.add_space(10.0);
                ui.vertical(|ui| {
                // ui.available_size();
                    ui.add_space(30.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Images").size(24.0));
                            if !self.image_sorting_modify.is_sorting {
                                let ordering_button = ui.button(egui::RichText::new(format!("{}", egui_phosphor::regular::ARROWS_DOWN_UP)).size(12.0).color(Color32::WHITE));
                                if ordering_button.clicked() {
                                    self.image_sorting_modify.is_sorting = true; 
                                }
                            };
                        });

                        let image_preview_scroll_area = ScrollArea::vertical().max_height(300.0);
                           
                        image_preview_scroll_area.show(ui, |ui| {
                            ui.set_width(350.);
                            ui.set_height(300.0);
                            let image_preview_cover_size = Vec2 {x: 250., y: 60.};
                            let margin: f32 = 2.5;
                            let padding: f32 = 5.;
                            for (idx, image_preview) in self.output_textures_vec.clone().iter().enumerate() {
                                ui.add_space(margin);
                                ui.horizontal(|ui| {

                                    let (image_cover_rect, _) = ui.allocate_exact_size(image_preview_cover_size, Sense::click());
                                    
                                    ui.painter().rect_filled(image_cover_rect, 5., Color32::from_rgb(255, 255, 255));
                                    let frame_width = image_cover_rect.width() - padding * 2.0;
                                    let frame_height = image_cover_rect.height() - padding * 2.0;
                                    let frame_ratio = frame_width / frame_height;
                                    let txt_ratio = image_preview.image_ratio;
                                
                                    
                                    let scaled = if txt_ratio > frame_ratio {
                                        frame_width / image_preview.texture_handle.as_ref().unwrap().size_vec2().x as f32
                                    } else {
                                        frame_height / image_preview.texture_handle.as_ref().unwrap().size_vec2().y as f32
                                    };
                                    
                                    let img_preview_rect = egui::Rect::from_center_size(
                                        Pos2 {x: padding * 5. + image_cover_rect.min.x + image_preview.texture_handle.as_ref().unwrap().size_vec2().x * scaled / 2.0, y: image_cover_rect.center().y},
                                        Vec2 { x: image_preview.texture_handle.as_ref().unwrap().size_vec2().x * scaled, y: image_preview.texture_handle.as_ref().unwrap().size_vec2().y as f32 * scaled }
                                    );
    
                                    let preview_uv = egui::Rect::from_min_max(
                                        Pos2 { x: 0.0, y: 0.0 }, 
                                        Pos2 { x: 1.0, y: 1.0 }
                                    );
                                    ui.painter().image(image_preview.texture_handle.as_ref().unwrap().id(), img_preview_rect, preview_uv, Color32::WHITE);
                                    ui.horizontal_centered(|ui| {
                                        ui.vertical_centered_justified( |ui| {
                                            
                                            
                                            let remove_button = ui.button(egui::RichText::new(format!("{}", egui_phosphor::regular::TRASH)).size(16.0));
                                            if remove_button.clicked(){
                                                self.remove_image(image_preview.texture_handle.as_ref().unwrap().id());
                                            }
                                        });
                                    });
                                    if let Some(active_img) = &self.active_image {
                                        if active_img.get_texture_id() == image_preview.texture_handle.as_ref().unwrap().id() {
                                            
                                            let points = Vec::from([
                                                Pos2::new(image_cover_rect.min.x + padding, image_cover_rect.center().y - padding * 2. ),
                                                Pos2::new(image_cover_rect.min.x + padding * 2., image_cover_rect.center().y),
                                                Pos2::new(image_cover_rect.min.x + padding, image_cover_rect.center().y + padding * 2.),
                                                Pos2::new(image_cover_rect.min.x + padding, image_cover_rect.center().y - padding * 2. ),
    
                                            ]);
                                            ui.painter().line(points, Stroke::new(1., Color32::BLACK));
                                        }
                                    };
                                    let click_res = ui.allocate_rect(image_cover_rect, Sense::click());
                                    if click_res.hovered() {
                                        ctx.set_cursor_icon(CursorIcon::PointingHand);
                                    }
                                    if click_res.clicked() {
                                        self.select_image(idx);
                                    }
                                });
                                

                                

                            }
                            
                        });
                        
                       
                        if self.image_sorting_modify.is_sorting {
                            self.display_sorting_images(ctx);
                        }
                    });
                    ui.separator();
                    egui::CollapsingHeader::new("Active Image Settings").show(ui, |ui| { 
                        if self.active_image_settings.croped_modified.is_modifying {
                            ui.label("Unable to scale while cropping");
                        } else {
                            let scale_slider = egui::Slider::new(&mut self.active_image_settings.transforms.scale, 0.1..=10.).text("Scale");
                            // scale_slider
                            ui.add(scale_slider);

                        }
                        ui.add(egui::Slider::new(&mut self.active_image_settings.transforms.opacity, 0.0..=1.).text("Opacity"));
                        // ui.add(egui::Slider::new(&mut self.active_image_settings.transforms.rotation, -180.0..=180.0).text("Rotation"));
                        egui::CollapsingHeader::new("Position").show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.active_image_settings.transforms.pos.x).speed(1.).prefix("X: "));
                                ui.add(egui::DragValue::new(&mut self.active_image_settings.transforms.pos.y).speed(1.).prefix("Y: "));

                            });
                        })
                    });
                   
                    if self.active_image.is_some() {
                        if self.active_image_settings.croped_modified.is_modifying == false {
                            let crop_button = ui.button("Crop Image").on_hover_text("Crop the current image");
                            if crop_button.clicked() {
                                
                                
                                let max_scale = calc_max_scale(self.active_image_settings.transforms.size, self.main_image_settings.image_plot_rect.size()) / self.active_image_settings.transforms.original_scale.clone();
                                let crop = if let Some(crop) = self.active_image_settings.transforms.croped {

                                    let multiply =  max_scale.clone();
                                    Some(CropRect {
                                        top: crop.top * multiply,
                                        left: crop.left * multiply,
                                        bottom: crop.bottom * multiply,
                                        right: crop.right * multiply,
                                    })
                                } else {
                                    Some(CropRect::default())
                                };
                                self.active_image_settings.croped_modified = CropedImageModified::default();
                                self.active_image_settings.croped_modified.is_modifying = true;
                                self.active_image_settings.croped_modified.max_scale = max_scale;
                                self.active_image_settings.croped_modified.prev_modified = crop.clone();
                                self.active_image_settings.croped_modified.current_modified = crop.clone();
                            }
                        } else {
                            ui.horizontal(|ui| {
                                let cancel_button =ui.button("Cancel").on_hover_text("Cancel Croping");
                                let apply_button = ui.button("Apply").on_hover_text("Apply Croping");
                                let reset_button = ui.button("Reset").on_hover_text("Reset Croping");
                                if cancel_button.clicked() {
                                    self.active_image_settings.croped_modified.is_modifying = false;
                                }
                                if apply_button.clicked() {
                                    self.active_image_settings.croped_modified.is_modifying = false;
                                    self.update_croped_image_tranform();
                                }
                                if reset_button.clicked() {
                                    self.active_image_settings.croped_modified.current_modified = self.active_image_settings.croped_modified.prev_modified;
                                }
                            });
                        }
                        egui::CollapsingHeader::new("Uv Display").show(ui, |ui| {
                            let (uv_display_rect, _uv_display_response) = ui.allocate_exact_size(Vec2 { x: 300., y: 300. }, Sense::click());
                            ui.painter().rect_filled(uv_display_rect, 2., Color32::from_rgb(80, 80, 80));
                            let uv_rect = self.active_image_settings.props.uv_rect;
                            let display_scale = 0.5;
                            let draw_rect = egui::Rect {
                                min: Pos2 { 
                                    x: uv_display_rect.center().x + (uv_rect.min.x - 0.5) * uv_display_rect.width() * display_scale, 
                                    y: uv_display_rect.center().y + (uv_rect.min.y - 0.5) * uv_display_rect.height() * display_scale
                                },
                                max: Pos2 { 
                                    x: uv_display_rect.center().x + (uv_rect.max.x - 0.5) * uv_display_rect.width() * display_scale, 
                                    y: uv_display_rect.center().y + (uv_rect.max.y - 0.5) * uv_display_rect.height() * display_scale
                                }
    
                            };             
                            ui.painter().rect_filled(draw_rect, 0., Color32::from_rgb(200, 200, 200));
                        });
                       
                    }
                });
           
           
            
          
            });
        });
    }
}
