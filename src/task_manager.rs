/*
This file is part of Tuigotchi Health.

Tuigotchi Health is free software: you can redistribute it and/or
modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

Tuigotchi Health is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with Tuigotchi Health. If not, see
<https://www.gnu.org/licenses/>.
*/

use chrono::{DateTime, Duration, Local};

use crate::config::Config;
use crate::task::{Task, TaskType};

use color_eyre::Result;

pub struct TaskManager {
    tasks: Vec<Task>,
    task_threshold: Duration,
}

#[derive(Debug, PartialEq)]
pub struct TaskDue {
    pub ty: TaskType,
    pub when: DateTime<Local>,
}

#[derive(Debug, Default)]
pub struct Tasks {
    pub past: Vec<TaskDue>,
    pub current: Vec<TaskDue>,
    pub upcoming: Vec<TaskDue>,
}

impl TaskManager {
    pub fn new(config: &mut Config) -> Result<Self> {
        Ok(Self {
            tasks: std::mem::take(&mut config.tasks),
            task_threshold: Duration::from_std(config.task_timeout)?,
        })
    }

    pub fn tasks(&self, now: DateTime<Local>) -> Result<Tasks> {
        let mut tasks = Tasks {
            past: vec![],
            current: vec![],
            upcoming: vec![],
        };

        for task in &self.tasks {
            let task_due = TaskDue {
                ty: task.ty().clone(),
                // We actually want to find the "next instance" in
                // relation to when it was last done, rather than now;
                // this gives the time when the task *should* be done,
                // or should have been done
                when: task.schedule().next_instance(task.last_done)?,
            };

            if task_due.when > now {
                tasks.upcoming.push(task_due);
            } else if now - task_due.when < self.task_threshold {
                tasks.current.push(task_due);
            } else {
                tasks.past.push(task_due);
            }
        }

        Ok(tasks)
    }

    pub fn complete_tasks(&mut self, ty: &TaskType, now: DateTime<Local>) {
        self.tasks
            .iter_mut()
            .filter(|t| t.ty() == ty)
            .for_each(|t| Task::complete(t, now));
    }
}
