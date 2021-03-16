pub use self::{
    blob::{Blob, BlobId, BlobType, MaybeBlob},
    contained_piece::ContainedPiece,
    media_type::MediaType,
    piece::{Piece, PieceId},
    source_type::SourceType,
    tag::Tag,
    tag_category::TagCategory,
};
use self::{tag::TagId, tag_category::TagCategoryId};
use piece::PieceCommand;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    ops::Add,
};
use table::{Table, TableCommand};
use undo::Command;

mod blob;
mod command;
mod contained_piece;
mod media_type;
mod piece;
mod source_type;
mod table;
mod tag;
mod tag_category;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Db {
    pub pieces: Table<Piece>,
    pub blobs: Table<Blob>,
    pub tags: Table<Tag>,
    pub tag_categories: Table<TagCategory>,

    pub media: BTreeSet<(PieceId, BlobId)>,
    pub piece_tags: BTreeSet<(PieceId, TagId, Option<TagCategoryId>)>,
    pub blob_tags: BTreeSet<(BlobId, TagId, Option<TagCategoryId>)>,
}

pub enum DbCommand {
    Piece(TableCommand<Piece, PieceCommand>),

    Media {
        add: bool,
        item: (PieceId, BlobId),
    },
    PieceTags {
        add: bool,
        item: (PieceId, TagId, Option<TagCategoryId>),
    },
    BlobTags {
        add: bool,
        item: (BlobId, TagId, Option<TagCategoryId>),
    },
}

impl Command for DbCommand {
    type Target = Db;

    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self {
            DbCommand::Media { add, item } => {
                if *add {
                    target.media.insert(*item);
                } else {
                    target.media.remove(item);
                }
            }
            DbCommand::PieceTags { add, item } => {
                if *add {
                    target.piece_tags.insert(*item);
                } else {
                    target.piece_tags.remove(item);
                }
            }
            DbCommand::BlobTags { add, item } => {
                if *add {
                    target.blob_tags.insert(*item);
                } else {
                    target.blob_tags.remove(item);
                }
            }
            DbCommand::Piece(command) => command.apply(&mut target.pieces)?,
        }
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self {
            DbCommand::Media { add, item } => {
                if !*add {
                    target.media.insert(*item);
                } else {
                    target.media.remove(item);
                }
            }
            DbCommand::PieceTags { add, item } => {
                if !*add {
                    target.piece_tags.insert(*item);
                } else {
                    target.piece_tags.remove(item);
                }
            }
            DbCommand::BlobTags { add, item } => {
                if !*add {
                    target.blob_tags.insert(*item);
                } else {
                    target.blob_tags.remove(item);
                }
            }
            DbCommand::Piece(command) => command.undo(&mut target.pieces)?,
        }
        Ok(())
    }
}
