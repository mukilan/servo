/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use compositing_traits::CrossProcessCompositorApi;
use ipc_channel::ipc::IpcSender;
use log::debug;
use malloc_size_of::MallocSizeOfOps;
use malloc_size_of_derive::MallocSizeOf;
use pixels::{CorsStatus, Image as RasterImage, ImageMetadata};
use profile_traits::mem::Report;
use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};
use webrender_api::units::DeviceIntSize;

use crate::FetchResponseMsg;
use crate::request::CorsSettings;

// ======================================================================
// Aux structs and enums.
// ======================================================================

pub type VectorImageId = PendingImageId;
// Represents either a raster image for which the pixel data is available
// or a vector image for which only the natural dimensions are available
// and thus require an further rasterization step.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum Image {
    Raster(#[conditional_malloc_size_of] Arc<RasterImage>),
    Vector(ImageMetadata, VectorImageId, CorsStatus),
}

impl Image {
    pub fn metadata(&self) -> ImageMetadata {
        match self {
            Image::Vector(metadata, ..) => *metadata,
            Image::Raster(image) => ImageMetadata {
                width: image.width,
                height: image.height,
            },
        }
    }

    pub fn cors_status(&self) -> CorsStatus {
        match self {
            Image::Vector(_, _, cors_status) => *cors_status,
            Image::Raster(image) => image.cors_status,
        }
    }

    pub fn as_raster_image(&self) -> Option<Arc<RasterImage>> {
        match self {
            Image::Raster(image) => Some(image.clone()),
            Image::Vector(..) => None,
        }
    }
}

/// Indicating either entire image or just metadata availability
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable {
        image: Image,
        url: ServoUrl,
        is_placeholder: bool,
    },
    MetadataAvailable(ImageMetadata, PendingImageId),
}

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ImageLoadListener {
    pipeline_id: PipelineId,
    pub id: PendingImageId,
    sender: IpcSender<ImageCacheMessage>,
}

impl ImageLoadListener {
    pub fn new(
        sender: IpcSender<ImageCacheMessage>,
        pipeline_id: PipelineId,
        id: PendingImageId,
    ) -> ImageLoadListener {
        ImageLoadListener {
            pipeline_id,
            sender,
            id,
        }
    }

    pub fn respond(&self, response: ImageResponse) {
        debug!("Notifying listener");
        // This send can fail if thread waiting for this notification has panicked.
        // That's not a case that's worth warning about.
        // TODO(#15501): are there cases in which we should perform cleanup?
        let msg = PendingImageResponse {
            pipeline_id: self.pipeline_id,
            response,
            id: self.id,
        };
        let _ = self
            .sender
            .send(ImageCacheMessage::NotifyPendingImageLoadStatus(msg));
    }
}

/// The returned image.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum ImageResponse {
    /// The requested image was loaded.
    Loaded(Image, ServoUrl),
    /// The request image metadata was loaded.
    MetadataLoaded(ImageMetadata),
    /// The requested image failed to load, so a placeholder was loaded instead.
    PlaceholderLoaded(#[conditional_malloc_size_of] Arc<RasterImage>, ServoUrl),
    /// Neither the requested image nor the placeholder could be loaded.
    None,
}

/// The unique id for an image that has previously been requested.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct PendingImageId(pub u64);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PendingImageResponse {
    pub pipeline_id: PipelineId,
    pub response: ImageResponse,
    pub id: PendingImageId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ImageCacheMessage {
    NotifyPendingImageLoadStatus(PendingImageResponse),
    VectorImageRasterizationCompleted(PipelineId, PendingImageId, DeviceIntSize),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum UsePlaceholder {
    No,
    Yes,
}

// ======================================================================
// ImageCache public API.
// ======================================================================

pub enum ImageCacheResult {
    Available(ImageOrMetadataAvailable),
    LoadError,
    Pending(PendingImageId),
    ReadyForRequest(PendingImageId),
}

pub trait ImageCache: Sync + Send {
    fn new(compositor_api: CrossProcessCompositorApi, rippy_data: Vec<u8>) -> Self
    where
        Self: Sized;

    fn memory_report(&self, prefix: &str, ops: &mut MallocSizeOfOps) -> Report;

    /// Definitively check whether there is a cached, fully loaded image available.
    fn get_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Image>;

    fn get_cached_image_status(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
        use_placeholder: UsePlaceholder,
    ) -> ImageCacheResult;

    fn rasterize_vector_image(
        &self,
        pipeline_id: PipelineId,
        image_id: VectorImageId,
        size: DeviceIntSize,
    ) -> Option<RasterImage>;

    fn add_rasterization_complete_listener(
        &self,
        pipeline_id: PipelineId,
        image_id: VectorImageId,
        size: DeviceIntSize,
        sender: IpcSender<ImageCacheMessage>,
    );

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, listener: ImageLoadListener);

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg);

    /// Create new image cache based on this one, while reusing the existing thread_pool.
    fn create_new_image_cache(
        &self,
        compositor_api: CrossProcessCompositorApi,
    ) -> Arc<dyn ImageCache>;
}
