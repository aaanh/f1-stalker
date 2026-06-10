#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStepId {
    Settings,
    Calendar,
    Drivers,
    Championship,
    Media,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStepStatus {
    Pending,
    Running,
    Done,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct BootStep {
    pub id: BootStepId,
    pub label: &'static str,
    pub status: BootStepStatus,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct BootState {
    pub active: bool,
    pub steps: Vec<BootStep>,
    pub media_total: u32,
    pub media_done: u32,
}

impl BootState {
    pub fn new() -> Self {
        Self {
            active: true,
            steps: vec![
                step(BootStepId::Settings, "Settings and pins"),
                step(BootStepId::Calendar, "Season calendar"),
                step(BootStepId::Drivers, "Driver roster"),
                step(BootStepId::Championship, "Championship standings"),
                step(BootStepId::Media, "Flags, logos and portraits"),
            ],
            media_total: 0,
            media_done: 0,
        }
    }

    pub fn step_mut(&mut self, id: BootStepId) -> &mut BootStep {
        self.steps
            .iter_mut()
            .find(|step| step.id == id)
            .expect("boot step exists")
    }

    pub fn start_step(&mut self, id: BootStepId, detail: impl Into<String>) {
        let step = self.step_mut(id);
        step.status = BootStepStatus::Running;
        step.detail = detail.into();
    }

    pub fn complete_step(&mut self, id: BootStepId, detail: impl Into<String>) {
        let step = self.step_mut(id);
        step.status = BootStepStatus::Done;
        step.detail = detail.into();
    }

    pub fn fail_step(&mut self, id: BootStepId, detail: impl Into<String>) {
        let step = self.step_mut(id);
        step.status = BootStepStatus::Failed;
        step.detail = detail.into();
    }

    pub fn skip_step(&mut self, id: BootStepId, detail: impl Into<String>) {
        let step = self.step_mut(id);
        step.status = BootStepStatus::Skipped;
        step.detail = detail.into();
    }

    pub fn begin_media(&mut self, total: u32) {
        self.media_total = total;
        self.media_done = 0;
        if total == 0 {
            self.complete_step(BootStepId::Media, "Nothing to download");
            return;
        }
        self.start_step(
            BootStepId::Media,
            format!("Downloading assets (0/{total})"),
        );
    }

    pub fn media_loaded(&mut self) {
        if self.media_total == 0 {
            return;
        }
        self.media_done = (self.media_done + 1).min(self.media_total);
        let detail = format!(
            "Downloading assets ({}/{})",
            self.media_done, self.media_total
        );
        if self.media_done >= self.media_total {
            self.complete_step(BootStepId::Media, "Assets ready");
        } else {
            self.start_step(BootStepId::Media, detail);
        }
    }

    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let weight = 1.0 / self.steps.len() as f32;
        let mut total = 0.0;

        for step in &self.steps {
            total += match step.status {
                BootStepStatus::Done | BootStepStatus::Skipped | BootStepStatus::Failed => weight,
                BootStepStatus::Running if step.id == BootStepId::Media => {
                    if self.media_total == 0 {
                        weight
                    } else {
                        weight * (self.media_done as f32 / self.media_total as f32)
                    }
                }
                BootStepStatus::Running => weight * 0.4,
                BootStepStatus::Pending => 0.0,
            };
        }

        total.clamp(0.0, 1.0)
    }

    pub fn current_label(&self) -> String {
        if let Some(step) = self
            .steps
            .iter()
            .find(|step| step.status == BootStepStatus::Running)
        {
            if step.detail.is_empty() {
                return step.label.to_string();
            }
            return format!("{} - {}", step.label, step.detail);
        }

        if let Some(step) = self.steps.iter().rev().find(|step| {
            matches!(
                step.status,
                BootStepStatus::Done | BootStepStatus::Skipped | BootStepStatus::Failed
            )
        }) {
            return step.detail.clone();
        }

        "Starting...".into()
    }

    pub fn try_finish(&mut self) {
        if !self.active {
            return;
        }

        let core_ready = [BootStepId::Calendar, BootStepId::Drivers, BootStepId::Championship]
            .iter()
            .all(|id| {
                self.steps
                    .iter()
                    .find(|step| step.id == *id)
                    .is_some_and(|step| {
                        matches!(
                            step.status,
                            BootStepStatus::Done
                                | BootStepStatus::Skipped
                                | BootStepStatus::Failed
                        )
                    })
            });

        let media_ready = self
            .steps
            .iter()
            .find(|step| step.id == BootStepId::Media)
            .is_some_and(|step| {
                matches!(
                    step.status,
                    BootStepStatus::Done | BootStepStatus::Skipped | BootStepStatus::Failed
                )
            });

        if core_ready && media_ready {
            self.active = false;
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

fn step(id: BootStepId, label: &'static str) -> BootStep {
    BootStep {
        id,
        label,
        status: BootStepStatus::Pending,
        detail: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reaches_one_when_all_steps_done() {
        let mut boot = BootState::new();
        boot.complete_step(BootStepId::Settings, "ok");
        boot.complete_step(BootStepId::Calendar, "ok");
        boot.complete_step(BootStepId::Drivers, "ok");
        boot.complete_step(BootStepId::Championship, "ok");
        boot.complete_step(BootStepId::Media, "ok");
        assert!((boot.progress() - 1.0).abs() < f32::EPSILON);
        boot.try_finish();
        assert!(!boot.active);
    }
}
