use undo::{Command, Merge};

pub struct BasicCommand<T> {
    pub from: Option<T>,
    pub to: T,
}

impl<T> BasicCommand<T> {
    fn set(value: T) -> Self {
        Self {
            from: None,
            to: value,
        }
    }
}

impl<T: Clone + Eq + PartialEq> Command for BasicCommand<T> {
    type Target = T;

    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        self.from = Some(target.clone());
        *target = self.to.clone();
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        *target = self.from.clone().unwrap();
        Ok(())
    }

    fn merge(&mut self, command: Self) -> Merge<Self> {
        if self.from.as_ref() == Some(&command.to) {
            Merge::Annul
        } else if command.from.is_some() {
            Merge::No(command)
        } else {
            self.to = command.to;
            Merge::Yes
        }
    }
}
