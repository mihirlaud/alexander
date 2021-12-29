use eframe::{egui, epi};

pub enum AppMode {
    Map,
    DiceRoller,
    Timers,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Map
    }
}

#[derive(PartialEq, Clone)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

impl Default for DiceType {
    fn default() -> Self {
        DiceType::D4
    }
}

fn get_dice_val(dice_type: DiceType) -> i32 {
    match dice_type {
        DiceType::D4 => 4,
        DiceType::D6 => 6,
        DiceType::D8 => 8,
        DiceType::D10 => 10,
        DiceType::D12 => 12,
        DiceType::D20 => 20,
        DiceType::D100 => 100,
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct TemplateApp {
    entities: Vec<Entity>,

    creating_entity: bool,
    new_entity_name: String,
    new_entity_hp: u32,
    new_entity_init: String,
    new_entity_init_mod: String,

    entity_removed: i32,
    
    with_timer: bool,
    encounter_started: bool,
    current_entity: usize,
    round: u32,

    mode: AppMode,

    dice_type: DiceType,
    dice_quantity: String,
    dice_modifier: String,
    dice_breakdown: Vec<(i32, i32)>,
    dice_roll_total: i32,
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Alexander"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        self.entity_removed = -1;

        if self.creating_entity {
            egui::Window::new("Create New Entity").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut self.new_entity_name);
                });

                ui.horizontal(|ui| {
                    ui.label("HP: ");
                    ui.add(egui::Slider::new(&mut self.new_entity_hp, 0..=300));
                });

                ui.horizontal(|ui| {
                    ui.label("Initiative: ");
                    ui.add(egui::TextEdit::singleline(&mut self.new_entity_init).desired_width(25.0));
                    if ui.button("Roll").clicked() {
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let mut init: i32 = rng.gen_range(1..=20) + self.new_entity_init_mod.parse::<i32>().unwrap_or_default();
                        if init < 1 {
                            init = 1;
                        }
                        self.new_entity_init = format!("{}", init);
                    }
                    ui.label(" d20 + ");
                    ui.add(egui::TextEdit::singleline(&mut self.new_entity_init_mod).desired_width(25.0));
                });

                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        self.creating_entity = false;
                        self.entities.push(
                            Entity::new(
                                self.new_entity_name.clone(), 
                                self.new_entity_hp, 
                                self.new_entity_init.parse::<u32>().unwrap_or(1),
                                self.new_entity_init_mod.parse::<i32>().unwrap_or(0)
                            )
                        );
                        self.entities.sort();

                        self.new_entity_name = "".to_string();
                        self.new_entity_hp = 0;
                        self.new_entity_init = "".to_string();
                        self.new_entity_init_mod = "".to_string();
                    }

                    if ui.button("Cancel").clicked() {
                        self.creating_entity = false;
                        self.new_entity_name = "".to_string();
                        self.new_entity_hp = 0;
                    }
                });
                
            });
        }

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("New").clicked() {
                        self.entities.clear();
                        self.creating_entity = false;

                        self.new_entity_name = "".to_string();
                        self.new_entity_hp = 0;
                        self.new_entity_init = "".to_string();
                        self.new_entity_init_mod = "".to_string();

                        self.with_timer = false;
                        self.encounter_started = false;
                        self.current_entity = 0;
                        self.round = 1;
                    }

                    if ui.button("Export").clicked() {}
                    if ui.button("Import").clicked() {}
                });

                ui.separator();

                if ui.button("Map").clicked() {
                    self.mode = AppMode::Map;
                }

                if ui.button("Dice Roller").clicked() {
                    self.mode = AppMode::DiceRoller;
                }

                if ui.button("Timers").clicked() {
                    self.mode = AppMode::Timers;
                }
            });
        });

        egui::SidePanel::left("entity-list").width_range(100.0..=300.0).default_width(200.0).show(ctx, |ui| {
            ui.add_space(5.0);

            ui.vertical_centered(|ui| {
                if ui.button("Add Entity").clicked() {
                    self.creating_entity = true;
                }
            });
                

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, entity) in self.entities.clone().iter().rev().enumerate() {
                    ui.vertical(|ui| {
                        if self.encounter_started && i == self.current_entity {
                            ui.add(egui::Label::new(entity.get_name()).strong().text_color(egui::Color32::GREEN));
                        } else {
                            ui.strong(entity.get_name());
                        }
                        ui.label(format!("HP: {}", entity.get_hp()));
    
                        ui.horizontal(|ui| {
                            ui.label(format!("Init: {}", entity.get_init()));
                            ui.add(egui::Label::new(format!(
                                "({} + {})", 
                                entity.get_init() as i32 - entity.get_dex_mod(), 
                                entity.get_dex_mod()
                            )).italics());
                        });
    
                        if ui.add(egui::Button::new("Remove").text_color(egui::Color32::RED)).clicked() {
                            self.entity_removed = i as i32;
                        }
                            
                    });
                    ui.separator();
                }
            });
            
        });

        egui::SidePanel::right("control-panel").width_range(100.0..=300.0).default_width(200.0).show(ctx, |ui| {
            ui.add_space(5.0);

            ui.vertical_centered(|ui| {
                let button_text = if self.encounter_started { "Stop Encounter" } else { "Start Encounter" };
                if ui.button(button_text).clicked() {
                    if !self.entities.is_empty() {
                        self.encounter_started = !self.encounter_started;
                        self.current_entity = 0;
                        self.round = 1;
                    } else {
                        // TODO: add warning about starting without entities
                    }
                }
                // ui.checkbox(&mut self.with_timer, "With timer");
                // TODO: add turn timer
            });

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            if self.encounter_started {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("Round {}", self.round));
                    ui.add_space(5.0);
                    if ui.button("Advance Turn").clicked() {
                        self.current_entity += 1;
                        if self.current_entity >= self.entities.len() {
                            self.current_entity = 0;
                            self.round += 1;
                        }
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Alexander");
            ui.label("This is a tool made for game masters to run RPG encounters more smoothly and efficiently.");
            
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            match self.mode {
                AppMode::Map => {
                    ui.heading("Map");
                }
                AppMode::DiceRoller => {
                    ui.heading("Dice Roller");

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.label("Select Dice: ");

                        ui.radio_value(&mut self.dice_type, DiceType::D4, "d4");
                        ui.radio_value(&mut self.dice_type, DiceType::D6, "d6");
                        ui.radio_value(&mut self.dice_type, DiceType::D8, "d8");
                        ui.radio_value(&mut self.dice_type, DiceType::D10, "d10");
                        ui.radio_value(&mut self.dice_type, DiceType::D12, "d12");
                        ui.radio_value(&mut self.dice_type, DiceType::D20, "d20");
                        ui.radio_value(&mut self.dice_type, DiceType::D100, "d100");
                    });
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("Quantity: ");
                        ui.add(egui::TextEdit::singleline(&mut self.dice_quantity).desired_width(25.0));
                        ui.label("Modifier: ");
                        ui.add(egui::TextEdit::singleline(&mut self.dice_modifier).desired_width(25.0));
                    });

                    ui.add_space(10.0);
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Roll").clicked() {
                            self.dice_roll_total = 0;
                            self.dice_breakdown.clear();

                            for _n in 0..self.dice_quantity.parse::<u32>().unwrap_or(0) {
                                use rand::Rng;
                                let mut rng = rand::thread_rng();

                                let roll = rng.gen_range(1 ..= get_dice_val(self.dice_type.clone()));

                                let modifier = self.dice_modifier.parse::<i32>().unwrap_or(0);

                                self.dice_breakdown.push((roll, modifier));

                                self.dice_roll_total += roll + modifier;
                            }
                        }
                    });
                    ui.add_space(10.0);

                    ui.label(format!("Total: {}", self.dice_roll_total));

                    ui.add_space(5.0);
                    if !self.dice_breakdown.is_empty() {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.strong("Breakdown: ");
                            ui.horizontal(|ui| {
                                for n in (0..self.dice_breakdown.len()).step_by(10) {
                                    ui.vertical(|ui| {
                                        for i in n .. n + 10 {
                                            if i > self.dice_breakdown.len() - 1 {
                                                break;
                                            }
                                            let (roll, modifier) = self.dice_breakdown.get(i).unwrap();
                                            let crit_val = get_dice_val(self.dice_type.clone());

                                            ui.horizontal(|ui| {
                                                if roll == &crit_val {
                                                    ui.add(egui::Label::new(format!("•  {} ", roll + modifier)).text_color(egui::Color32::GREEN));
                                                    ui.add(egui::Label::new(format!("({} + {})", roll, modifier)).text_color(egui::Color32::GREEN).italics());
                                                } else if roll == &1 {
                                                    ui.add(egui::Label::new(format!("•  {} ", roll + modifier)).text_color(egui::Color32::RED));
                                                    ui.add(egui::Label::new(format!("({} + {})", roll, modifier)).text_color(egui::Color32::RED).italics());
                                                } else {
                                                    ui.label(format!("•  {} ", roll + modifier));
                                                    ui.add(egui::Label::new(format!("({} + {})", roll, modifier)).italics());
                                                }
                                            });
                                        }
                                    });
                                    ui.separator();
                                }
                            });
                            // for (roll, modifier) in self.dice_breakdown.clone() {
                            //     ui.horizontal(|ui| {
                            //         ui.label(format!("•  {} ", roll + modifier));
                            //         ui.add(egui::Label::new(format!("({} + {})", roll, modifier)).italics());
                            //     });
                            // }
                        });
                    }
                    
                }
                AppMode::Timers => {
                    ui.heading("Timers");
                }
            }
            
            egui::warn_if_debug_build(ui);
        });

        if self.entity_removed >= 0 {
            self.entities.remove(self.entities.len() - 1 - self.entity_removed as usize);
        }
    }
}

#[derive(Hash, PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
struct Entity {
    init: u32,
    dex_mod: i32,
    name: String,
    hp: u32,
}

impl Entity {
    pub fn new(name: String, hp: u32, init: u32, dex_mod: i32) -> Self {
        Self {
            init,
            dex_mod,
            name,
            hp,
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_hp(&self) -> u32 {
        self.hp
    }

    fn get_init(&self) -> u32 {
        self.init
    }

    fn get_dex_mod(&self) -> i32 {
        self.dex_mod
    }

    fn set_hp(&mut self, hp: u32) {
        self.hp = hp;
    }
}