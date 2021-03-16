use crate::{command::BasicCommand, table::TableId};

use super::{media_type::MediaType, source_type::SourceType};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use undo::{Command, Merge};

pub type PieceId = TableId<Piece>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Piece {
    pub name: String,
    pub source_type: SourceType,
    pub media_type: MediaType,
    pub added: DateTime<Local>,
    pub base_price: Option<i64>,
    pub tip_price: Option<i64>,
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            name: "New Piece".to_string(),
            source_type: SourceType::Commission,
            media_type: MediaType::Image,
            added: chrono::Local::now(),
            base_price: None,
            tip_price: None,
        }
    }
}

pub enum PieceCommand {
    Name(BasicCommand<String>),
    SourceType(BasicCommand<SourceType>),
    MediaType(BasicCommand<MediaType>),
    Added(BasicCommand<DateTime<Local>>),
    BasePrice(BasicCommand<Option<i64>>),
    TipPrice(BasicCommand<Option<i64>>),
}

impl Command for PieceCommand {
    type Target = Piece;

    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self {
            PieceCommand::Name(command) => command.apply(&mut target.name),
            PieceCommand::SourceType(command) => command.apply(&mut target.source_type),
            PieceCommand::MediaType(command) => command.apply(&mut target.media_type),
            PieceCommand::Added(command) => command.apply(&mut target.added),
            PieceCommand::BasePrice(command) => command.apply(&mut target.base_price),
            PieceCommand::TipPrice(command) => command.apply(&mut target.tip_price),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self {
            PieceCommand::Name(command) => command.undo(&mut target.name),
            PieceCommand::SourceType(command) => command.undo(&mut target.source_type),
            PieceCommand::MediaType(command) => command.undo(&mut target.media_type),
            PieceCommand::Added(command) => command.undo(&mut target.added),
            PieceCommand::BasePrice(command) => command.undo(&mut target.base_price),
            PieceCommand::TipPrice(command) => command.undo(&mut target.tip_price),
        }
    }

    fn merge(&mut self, command: Self) -> Merge<Self> {
        match (self, command) {
            (PieceCommand::Name(inner), PieceCommand::Name(new)) => match inner.merge(new) {
                Merge::No(command) => Merge::No(PieceCommand::Name(command)),
                Merge::Yes => Merge::Yes,
                Merge::Annul => Merge::Annul,
            },
            (PieceCommand::Added(inner), PieceCommand::Added(new)) => match inner.merge(new) {
                Merge::No(command) => Merge::No(PieceCommand::Added(command)),
                Merge::Yes => Merge::Yes,
                Merge::Annul => Merge::Annul,
            },
            (PieceCommand::BasePrice(inner), PieceCommand::BasePrice(new)) => {
                match inner.merge(new) {
                    Merge::No(command) => Merge::No(PieceCommand::BasePrice(command)),
                    Merge::Yes => Merge::Yes,
                    Merge::Annul => Merge::Annul,
                }
            }
            (PieceCommand::TipPrice(inner), PieceCommand::TipPrice(new)) => {
                match inner.merge(new) {
                    Merge::No(command) => Merge::No(PieceCommand::TipPrice(command)),
                    Merge::Yes => Merge::Yes,
                    Merge::Annul => Merge::Annul,
                }
            }
            (_, command) => Merge::No(command),
        }
    }
}
