use chatgpt::config::ChatGPTEngine;
use chatgpt::config::ModelConfigurationBuilder;
use chatgpt::prelude::ChatGPT;
use chatgpt::prelude::Conversation;
use chatgpt::types::ChatMessage;
use chatgpt::types::Role;
use egui::Vec2;
use egui_notify::Toasts;
use std::format;
use std::println;
use std::sync::Arc;
use std::time::Duration;

use egui_extras::RetainedImage;
use tokio::sync::*;

use crate::settings;
use crate::settings::Settings;

pub struct App {
    conversation: Option<Arc<Mutex<Conversation>>>,
    pmt: String,
    history: Vec<ChatMessage>,
    ai_icon: RetainedImage,
    user_icon: RetainedImage,
    system_icon: RetainedImage,
    send_icon: RetainedImage,
    is_side_panel_expanded: bool,
    settings: Settings,
    toasts: Toasts,
    app_name: String,
    current_role: settings::Role,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, app_name: &str) -> Self {
        let config_path = confy::get_configuration_file_path(app_name, None);
        println!("config_path:{:#?}", config_path);
        let settings: Settings = confy::load(app_name, None).unwrap();
        let current_role = settings.role_list.get(0).unwrap().to_owned();

        println!("{:#?}", settings);

        Self {
            conversation: None,
            pmt: "".to_string(),
            history: Vec::new(),
            ai_icon: RetainedImage::from_image_bytes(
                "chatgpt_logo.jpeg",
                include_bytes!("../media/chatgpt_logo.jpeg"),
            )
            .unwrap(),
            user_icon: RetainedImage::from_image_bytes(
                "user.jpeg",
                include_bytes!("../media/user.png"),
            )
            .unwrap(),
            system_icon: RetainedImage::from_image_bytes(
                "send.png",
                include_bytes!("../media/system.png"),
            )
            .unwrap(),
            send_icon: RetainedImage::from_image_bytes(
                "send.png",
                include_bytes!("../media/Send.png"),
            )
            .unwrap(),
            is_side_panel_expanded: false,
            settings,
            toasts: Toasts::default(),
            app_name: app_name.to_owned(),
            current_role,
        }
    }

    fn create_conversation(&self) -> Conversation {
        println!("new conversation with role {:#?}", self.current_role);
        let url: &'static str = Box::leak(self.settings.api_url.clone().into_boxed_str());
        ChatGPT::new_with_config(
            self.settings.api_key.clone(),
            ModelConfigurationBuilder::default()
                .api_url(url)
                .temperature(1.0)
                .engine(ChatGPTEngine::Gpt35Turbo)
                .build()
                .unwrap(),
        )
        .unwrap()
        .new_conversation_directed(self.current_role.prompt.clone())
    }

    fn render_role_list(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.menu_image_button(self.ai_icon.texture_id(ctx), Vec2::splat(24.0), |ui| {
            for role in self.settings.role_list.iter() {
                let is_current_role = role.eq(&self.current_role);
                if ui
                    .selectable_label(is_current_role, role.name.clone())
                    .clicked()
                {
                    if !is_current_role {
                        self.current_role = role.clone();
                        self.conversation = None;
                        self.history.clear();
                        println!("new role!! {:#?}", role);
                    }
                    println!("current_role={:#?} ", self.current_role);
                    ui.close_menu();
                }
            }
        });
    }

    ///to avoid the historical messages disappear before the response be received from openai
    fn sync_new_message(&mut self) -> bool {
        match self.conversation.clone() {
            Some(conversation) => {
                if let Ok(conversation) = conversation.try_lock_owned() {
                    if self.history.len() <= conversation.history.len()
                        && self.history.last() != conversation.history.last()
                    {
                        self.history = conversation.history.clone();
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    fn render_side_panel_handle(&mut self, ui: &mut egui::Ui) {
        let mut panel_handle = "》";
        if self.is_side_panel_expanded {
            panel_handle = "《";
        }
        if ui.small_button(panel_handle).clicked() {
            self.is_side_panel_expanded = !self.is_side_panel_expanded;
        }
    }

    fn render_history_messages(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                //wether or not scroll to new message
                let need_scroll = self.sync_new_message();

                for msg in self.history.clone().iter() {
                    match msg.role {
                        Role::System => {
                            // ignore role setting message
                            if msg.content == self.current_role.prompt {
                                continue;
                            }

                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                self.system_icon.show_size(ui, Vec2::splat(24.0));

                                let resp = ui
                                    .add(
                                        egui::Label::new(msg.content.clone())
                                            .wrap(true)
                                            .sense(egui::Sense::click()),
                                    )
                                    .on_hover_text_at_pointer("📋 点击复制");

                                if need_scroll {
                                    resp.scroll_to_me(None);
                                }

                                if resp.clicked() {
                                    ui.output_mut(|o| {
                                        o.copied_text = msg.content.clone();
                                        self.toasts
                                            .success("复制成功")
                                            .set_duration(Some(Duration::from_secs(1)));
                                    });
                                }
                            });
                        }
                        Role::Assistant => {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                if ui
                                    .add(egui::widgets::ImageButton::new(
                                        self.ai_icon.texture_id(ui.ctx()),
                                        Vec2::splat(24.0),
                                    ))
                                    .on_hover_text("❌点击删除")
                                    .clicked()
                                {
                                    println!("current history message have benn cleaned!");
                                    self.history.clear();
                                    self.conversation = None;
                                    println!("history size: {}", self.history.len());
                                    self.toasts
                                        .success("当前会话已重置！")
                                        .set_duration(Some(Duration::from_secs(1)));
                                }
                                let mut content = egui::text::LayoutJob::default();

                                content.append(
                                    &msg.content,
                                    0.0,
                                    egui::TextFormat {
                                        ..Default::default()
                                    },
                                );

                                let resp = ui
                                    .add(
                                        egui::Label::new(content)
                                            .wrap(true)
                                            .sense(egui::Sense::click()),
                                    )
                                    .on_hover_text_at_pointer("📋 点击复制");

                                if need_scroll {
                                    resp.scroll_to_me(None);
                                }

                                if resp.clicked() {
                                    ui.output_mut(|o| {
                                        o.copied_text = msg.content.clone();
                                        self.toasts
                                            .success("复制成功")
                                            .set_duration(Some(Duration::from_secs(1)));
                                    });
                                }
                            });
                        }
                        Role::User => {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                if ui
                                    .add(egui::widgets::ImageButton::new(
                                        self.user_icon.texture_id(ui.ctx()),
                                        Vec2::splat(24.0),
                                    ))
                                    .on_hover_text("❌点击删除")
                                    .clicked()
                                {
                                    println!("current history message have benn cleaned!");
                                    self.history.clear();
                                    self.conversation = None;
                                    println!("history size: {}", self.history.len());
                                    self.toasts
                                        .success("当前会话已重置！")
                                        .set_duration(Some(Duration::from_secs(1)));
                                }

                                let resp = ui
                                    .add(
                                        egui::Label::new(msg.content.clone())
                                            .wrap(true)
                                            .sense(egui::Sense::click()),
                                    )
                                    .on_hover_text_at_pointer("📋 点击复制");

                                if need_scroll {
                                    resp.scroll_to_me(None);
                                }

                                if resp.clicked() {
                                    ui.output_mut(|o| {
                                        o.copied_text = msg.content.clone();
                                        self.toasts
                                            .success("复制成功")
                                            .set_duration(Some(Duration::from_secs(1)));
                                    });
                                }
                            });
                        }
                    }
                    ui.separator();
                    ui.add_space(22_f32);
                }
            });
        });
    }

    fn render_history_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                self.render_side_panel_handle(ui);
                self.render_history_messages(ui);
            });
            ui.separator();
        });
    }

    fn render_spinner_if_necessary(&mut self, ui: &mut egui::Ui) {
        // If we can acquire the lock successfully, means there's no prompt submitting thread exist currently
        // namely: the App is waiting for user's inputing
        let is_waiting_for_ai = match self.conversation.clone() {
            Some(conversation) => !conversation.try_lock().is_ok(),
            None => false,
        };

        if is_waiting_for_ai {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.spinner();
                },
            );
            return;
        }
    }

    fn render_input_box(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            //ui.add_space(2_f32);

            ui.horizontal(|ui| {
                self.render_role_list(ctx, ui);

                self.render_spinner_if_necessary(ui);
                let prompt_text_edit = egui::TextEdit::multiline(&mut self.pmt)
                    .desired_width(f32::INFINITY)
                    .desired_rows(1)
                    .margin(egui::Vec2::splat(24_f32))
                    .hint_text("回车键发送");

                let resp = ui.add(prompt_text_edit);
                if !self.is_side_panel_expanded {
                    resp.request_focus();
                }

                // register the event of pressing Enter to send the message
                if resp.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && self.pmt.trim().len() > 1
                {
                    if self.conversation.is_none() {
                        self.conversation = Some(Arc::new(Mutex::new(self.create_conversation())));
                    }

                    self.history.push(ChatMessage {
                        role: Role::User,
                        content: self.pmt.trim().to_owned(),
                    });

                    tokio::spawn(App::submit_prompt(
                        ctx.clone(),
                        self.conversation.clone().unwrap().clone(),
                        self.pmt.trim().to_owned(),
                    ));
                    self.pmt.clear();
                }
            });

            //ui.add_space(10_f32);
        });
    }

    fn render_side_panel(&mut self, ctx: &egui::Context) {
        let side_panel_width = ctx.screen_rect().max.x / 4.0;
        egui::SidePanel::left("side_panel")
            .show_separator_line(true)
            .exact_width(side_panel_width)
            .show_animated(ctx, self.is_side_panel_expanded, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("API_KEY ");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.settings.api_key)
                                .desired_width(side_panel_width * 0.9),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("API_URL ");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.settings.api_url)
                                .desired_width(side_panel_width * 0.9),
                        );
                    });

                    ui.add_space(22.0);
                    if ui.button("保存").clicked() {
                        println!("ready to save settings:{:#?}", self.settings);
                        match confy::store(self.app_name.as_str(), None, self.settings.clone()) {
                            Err(err) => {
                                self.toasts
                                    .error(format!("保存失败！（{err}）"))
                                    .set_duration(None);
                            }
                            _ => {
                                self.toasts
                                    .warning("为了使新设置生效，请手动重启本应用！")
                                    .set_duration(None);
                            }
                        };
                    }
                });
            });
    }

    fn render_notification(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }

    async fn submit_prompt(
        ctx: egui::Context,
        conversation: Arc<Mutex<Conversation>>,
        pmt: String,
    ) {
        println!("====[send message:{:#?}]=====", pmt);

        let mut conversation = conversation.lock().await;
        if let Err(e) = conversation.send_message(pmt).await {
            conversation.history.push(ChatMessage {
                role: Role::System,
                content: format!("{:#?}", e),
            });
            println!("{:#?}", e);
        }
        ctx.request_repaint();
    }
}

//main loop running for ever
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_side_panel(ctx);
        self.render_input_box(ctx);

        self.render_history_panel(ctx);

        self.render_notification(ctx);
    }
}
