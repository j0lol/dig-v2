use bevy_ecs::{schedule::{IntoSystemConfigs, Schedule}, world::World};
use macroquad::window::next_frame;


pub enum ScheduleLabel_ {
    Startup,
    Update,
    PreUpdate,
    PostUpdate
}

#[derive(Default)]
pub struct App {
    startup_schedule: Schedule,
    update_schedule: Schedule,
    preupdate_schedule: Schedule,
    postupdate_schedule: Schedule,
    pub world: World
}

impl App {
    pub fn new() -> Self {
        App {
            world: World::new(),
            ..Default::default()
        }
    }
    pub async fn run(&mut self) -> ! {
        self.startup();
        loop {
            self.update();
            next_frame().await;
        }
    }
    pub fn startup(&mut self) {
        self.startup_schedule.run(&mut self.world);
    }
    pub fn update(&mut self) {
        self.preupdate_schedule.run(&mut self.world);
        self.update_schedule.run(&mut self.world);
        self.postupdate_schedule.run(&mut self.world);
    }
    pub fn add_systems<M>(
        &mut self,
        schedule: ScheduleLabel_,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        match schedule {
            ScheduleLabel_::Startup => self.startup_schedule.add_systems(systems), 
            ScheduleLabel_::Update => self.update_schedule.add_systems(systems),
            ScheduleLabel_::PreUpdate => self.preupdate_schedule.add_systems(systems),
            ScheduleLabel_::PostUpdate => self.postupdate_schedule.add_systems(systems),
        };
        
        self
    }
    pub fn add_plugin(
        &mut self,
        plugin: impl Plugin
    ) -> &mut Self {
        plugin.build(self);
        self
    }
}

pub trait Plugin: Send + Sync {
    /// Configures the [`App`] to which this plugin is added.
    fn build(&self, app: &mut App);
}