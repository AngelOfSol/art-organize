use imgui::{im_str, TreeNode, Ui};

use super::{GuiHandle, GuiView};

#[derive(Debug)]
pub struct Help;

impl GuiView for Help {
    fn update(&self, _: &super::GuiHandle) {}
    fn draw_main(&mut self, _: &GuiHandle, _: &super::InnerGuiState, ui: &imgui::Ui<'_>) {
        ui.text(im_str!("Help Screen"));

        sub_heading(ui, "Basics", || {
            items(
                ui,
                &[
                    "Most UI elements have tooltips, hover over them to find out!",
                    "Most screens have buttons to click to bring you to further detail.",
                    "Double-right click to go back to the previous screen.",
                ],
            );
        });
        sub_heading(ui, "Design", || {
            items(
                ui,
                &[
                    "ArtOrganize is an application to help you collate and annotate your \
                        commissions with metadata you might find useful.",
                    "Instead of tagging and annotating images directly, ArtOrganize groups \
                        images into 'pieces'.",
                    "A piece is just a collection of images, with additional metadata.",
                    "Each image within the piece itself, can have a small amount of \
                        metadata, but the intention is that most of the metadata is \
                        described in the piece containing the image.",
                    "The intention here, is to allow you to organize all of the drafts, raws\
                        , and variants with the same data that the final piece is annotated with.",
                    "The following is not implemented yet:",
                    "In addition to the standard metadata that a piece can have, \
                        you may also tag your images.",
                    "Tags have a name and some additional metadata, \
                        like a description, or a category.",
                    "Categories let you organize tags by type, and any tags in a category \
                        will inherit the categories color for recognition purposes.",
                ],
            );
        });
        sub_heading(ui, "Editing", || {
            items(
                ui,
                &[
                    "When editing text fields, you must hit Enter to confirm your change.\
                        If you don't your changes won't be confirmed.",
                    "Otherwise changes are saved automatically as you edit your collection.",
                ],
            );
        });

        sub_heading(ui, "Gallery", || {
            items(ui, &[
                "The gallery shows an un-filtered and un-sorted list of your pieces.",
                "A blob with type 'Canon' will be displayed as a thumbnail for each piece.",
                "The tag explorer on the left will show tags for the pieces currently on screen.",
                "Hovering over a piece brings up a small informational tooltip.",
                "Clicking on a piece will let you view that piece along with its tags and blobs."
            ])
        });

        sub_heading(ui, "Blobs", || {
            items(
                ui,
                &["ArtOrganize refers to images as blobs. \
                    Future plans support other types of blobs."],
            );
            sub_heading(ui, "Fields", || {
                items(
                    ui,
                    &[
                        "File Name: A title to give the blob, which does not have to be unique.  \
                            It will be part of the file name on the local filesystem.",
                        "Hash: A hash of the blob's raw file content, \
                            used for deduplication purposes.",
                        "Blob Type: One of Canon, Variant, Raw, or Draft.  \
                            The Canon type is used to choose thumbnails, but \
                            otherwise used for your own organizational purposes.",
                        "Date Added: The date which this blob was added to your \
                            collection.",
                    ],
                );
            });

            sub_heading(ui, "Adding Blobs", || {
                sub_heading(ui, "Methods", || {
                    items(
                        ui,
                        &[
                            "Dragging and dropping file(s) into the window will\
                                add them to the piece on screen.  They will \
                                have the blob type of where you dragged it into.",
                            "Clicking on the big '+' button in any category \
                                will let you select a list of files to add to the\
                                piece.  They will be assigned to that blob type.",
                        ],
                    );
                });
                items(
                    ui,
                    &[
                        "When you add a new blob to a piece, \
                            the file added will be copied, \
                            but NOT deleted, from its \
                            original location into the database's root directory.",
                        "The blobs file name on the local file system will be \
                        it's database filename pre-pended with an autogenerated \
                        blob ID.",
                        "New blobs will have today as their default 'Date Added' value.",
                    ],
                );
            });
            sub_heading(ui, "Editing Blobs", || {
                items(
                    ui,
                    &["If you edit the file name of a blob, it will automatically rename the file \
                    on the local filesystem."],
                );
            });

            sub_heading(ui, "Deleting Blobs", || {
                items(
                    ui,
                    &[
                        "Deleting a blob will remove it from the piece and \
                            from the database itself.",
                        "It will NOT remove the file from the local file system.  \
                            Instead, if you can go to File > Clean Blobs to move \
                            all files that are not in the database to the local \
                            trash or recycle bin.",
                    ],
                );
            });
        });

        sub_heading(ui, "Pieces", || {
            items(
                ui,
                &[
                    "A piece is a collection of blobs, with \
                        some associated data and a list of tags.",
                    "Each blob will either by the canonical image, or a draft, raw, or variant.",
                    "The first image under the 'Canon' heading will be the thumbnail \
                        or otherwise chosen image when referring to a piece.",
                    "To create a new piece, goto Data > New Piece.  This will \
                        create and show an empty piece.",
                    "New pieces will have today as their default 'Date Added' value.",
                    "You may optionally set a base price or a tip price value, \
                    rounded to the nearest whole unit of currency.",
                ],
            );
            sub_heading(ui, "Fields", || {
                items(
                    ui,
                    &[
                        "Name: A title for the piece, which does not have to be unique.",
                        "Source Type: The origin of the piece, whether it was commissioned, \
                            drawn as fanart, or is from official sources.",
                        "Media Type: Image or Text (unsupported).",
                        "Date Added: The date which this piece was added to your \
                            collection.",
                        "Links: A list of hyperlinks associated with your piece. \
                            generally places where the image can be found on social media.",
                        "Base Price: For art that you paid for, the base price of the piece.",
                        "Tip Price: Any additional money given as a tip.",
                    ],
                );
            });

            sub_heading(ui, "Adding Pieces", || {
                items(
                    ui,
                    &[
                        "Go to File > New Piece.  An empty piece will be created, \
                        and you will be brought to edit it",
                        "New pieces will have today as their default 'Date Added' value.",
                    ],
                );
            });

            sub_heading(ui, "Deleting Pieces", || {
                items(
                    ui,
                    &[
                        "Deleting a piece will remove it from the database itself.",
                        "It will not delete any associated blobs, for now.  \
                            The associated blobs will be dangling for now, \
                            but using File > Clean Blobs will purge unassociated \
                            blobs from the database.",
                    ],
                );
            });
        });
    }

    fn draw_explorer(&mut self, _: &GuiHandle, _: &super::InnerGuiState, _: &imgui::Ui<'_>) {}
}

fn sub_heading<F: FnOnce()>(ui: &Ui<'_>, label: &str, f: F) {
    TreeNode::new(&im_str!("{}", label))
        .label(&im_str!("{}", label))
        .build(ui, f);
}

fn items(ui: &Ui<'_>, labels: &[&str]) {
    for label in labels {
        wrapped_bullet(ui, label)
    }
}

fn wrapped_bullet(ui: &Ui<'_>, s: &str) {
    ui.bullet();
    ui.same_line();
    ui.text_wrapped(&im_str!("{}", s));
}
