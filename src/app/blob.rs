use db::{BlobId, Db};
use imgui::{im_str, Ui};

pub fn view(blob_id: BlobId, db: &Db, ui: &Ui<'_>) {
    let blob = &db[blob_id];
    ui.text_wrapped(&im_str!("File Name: {}", blob.file_name));
    ui.text_wrapped(&im_str!("Blob Type: {}", blob.blob_type));
    ui.text_wrapped(&im_str!("Hash: {:x}", blob.hash));
    ui.text(im_str!(
        "Date Added: {}",
        blob.added.format("%-m/%-d/%-Y %-H:%-M %P")
    ));
}
